//! Checksum and structural validators.
//!
//! Regexes are good at finding *candidates* and bad at deciding whether a
//! candidate is real. This module applies the actual rule for each type:
//! Luhn for card numbers, mod-97 for IBANs, group counting for IPv6.
//!
//! That is what keeps dates (`2026-08-30`), version strings (`10.20.30`) and
//! arbitrary digit runs out of the results.

/// Luhn (mod-10) checksum used by payment card numbers.
///
/// Separators in `raw` are ignored. The digit count must land in the
/// 13–19 range mandated by ISO/IEC 7812.
pub fn luhn_ok(raw: &str) -> bool {
    let digits: Vec<u32> = raw
        .bytes()
        .filter(|b| b.is_ascii_digit())
        .map(|b| u32::from(b - b'0'))
        .collect();

    if !(13..=19).contains(&digits.len()) {
        return false;
    }

    let mut sum = 0u32;
    for (i, d) in digits.iter().rev().enumerate() {
        // Double every second digit from the right.
        let d = if i % 2 == 1 {
            let doubled = d * 2;
            if doubled > 9 { doubled - 9 } else { doubled }
        } else {
            *d
        };
        sum += d;
    }

    sum % 10 == 0
}

/// Sanity check on the issuer identification number.
///
/// A random 16-digit run has a ~10% chance of passing Luhn by accident, so we
/// also require the prefix to belong to a real scheme. This is intentionally
/// permissive — it filters noise, it does not certify the card.
pub fn has_card_prefix(raw: &str) -> bool {
    // `raw` is filtered to ASCII digits, so byte slicing matches char bounds.
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let prefix = |from: usize, to: usize| -> Option<u32> {
        if to <= digits.len() {
            digits[from..to].parse().ok()
        } else {
            None
        }
    };

    if prefix(0, 1) == Some(4) {
        return true; // Visa
    }
    if matches!(prefix(0, 2), Some(34) | Some(37)) {
        return true; // American Express
    }
    if matches!(prefix(0, 2), Some(51..=55)) {
        return true; // Mastercard
    }
    if matches!(prefix(0, 4), Some(2221..=2720)) {
        return true; // Mastercard 2-series
    }
    if prefix(0, 4) == Some(6011) || prefix(0, 2) == Some(65) {
        return true; // Discover
    }
    if matches!(prefix(0, 3), Some(644..=649)) {
        return true; // Discover
    }
    if prefix(0, 2) == Some(35) {
        return true; // JCB
    }
    if matches!(prefix(0, 3), Some(300..=305)) || prefix(0, 4) == Some(3095) {
        return true; // Diners Club
    }

    matches!(prefix(0, 2), Some(36) | Some(38) | Some(39)) // Diners Club
}

/// Registered IBAN length by country code.
/// `None` means the country is unknown and only the 15–34 envelope applies.
fn iban_length(country: &str) -> Option<usize> {
    Some(match country {
        "AD" => 24, "AE" => 23, "AL" => 28, "AT" => 20, "AZ" => 28,
        "BA" => 20, "BE" => 16, "BG" => 22, "BH" => 22, "BR" => 29,
        "BY" => 28, "CH" => 21, "CR" => 22, "CY" => 28, "CZ" => 24,
        "DE" => 22, "DK" => 18, "DO" => 28, "EE" => 20, "EG" => 29,
        "ES" => 24, "FI" => 18, "FO" => 18, "FR" => 27, "GB" => 22,
        "GE" => 22, "GI" => 23, "GL" => 18, "GR" => 27, "GT" => 28,
        "HR" => 21, "HU" => 28, "IE" => 22, "IL" => 23, "IQ" => 23,
        "IS" => 26, "IT" => 27, "JO" => 30, "KW" => 30, "KZ" => 20,
        "LB" => 28, "LC" => 32, "LI" => 21, "LT" => 20, "LU" => 20,
        "LV" => 21, "LY" => 25, "MC" => 27, "MD" => 24, "ME" => 22,
        "MK" => 19, "MR" => 27, "MT" => 31, "MU" => 30, "NL" => 18,
        "NO" => 15, "PK" => 24, "PL" => 28, "PS" => 29, "PT" => 25,
        "QA" => 29, "RO" => 24, "RS" => 22, "SA" => 24, "SC" => 31,
        "SE" => 24, "SI" => 19, "SK" => 24, "SM" => 27, "ST" => 25,
        "SV" => 28, "TL" => 23, "TN" => 24, "TR" => 26, "UA" => 29,
        "VA" => 22, "VG" => 24, "XK" => 20,
        _ => return None,
    })
}

/// Structure + mod-97 (ISO 7064) validation for an IBAN candidate.
///
/// Accepts spaced (`DE89 3704 0044 0532 0130 00`) or unspaced input.
/// Only the canonical uppercase form is recognised — lowercase input is
/// treated as prose, which keeps the false-positive rate down.
pub fn is_valid_iban(raw: &str) -> bool {
    let s: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if !(15..=34).contains(&s.len()) {
        return false;
    }

    // Shape: AA99 followed by alphanumerics.
    let mut chars = s.chars();
    if !chars.next().is_some_and(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if !chars.next().is_some_and(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    if !chars.next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric()) {
        return false;
    }

    if let Some(expected) = iban_length(&s[..2]) {
        if s.len() != expected {
            return false;
        }
    }

    // Move the leading four characters to the end, expand letters to numbers
    // (A=10 … Z=35) and reduce modulo 97 incrementally to avoid big integers.
    let (head, tail) = s.split_at(4);
    let mut remainder = 0u32;
    for c in tail.chars().chain(head.chars()) {
        let value = match c.to_digit(10) {
            Some(v) => v,
            None => c as u32 - 'A' as u32 + 10,
        };
        remainder = if value < 10 {
            (remainder * 10 + value) % 97
        } else {
            (remainder * 100 + value) % 97
        };
    }

    remainder == 1
}

/// Strict dotted-quad parse. Rejects leading zeros (`01`) and octets above 255.
pub fn parse_ipv4(candidate: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = candidate.split('.').collect();
    if parts.len() != 4 {
        return None;
    }

    let mut octets = [0u8; 4];
    for (slot, part) in octets.iter_mut().zip(parts) {
        if part.is_empty() || part.len() > 3 {
            return None;
        }
        if !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if part.len() > 1 && part.starts_with('0') {
            return None;
        }
        *slot = part.parse::<u8>().ok()?;
    }

    Some(octets)
}

/// Structural validation for an IPv6 candidate produced by the regex.
///
/// Handles the `::` compression rules, an embedded IPv4 tail
/// (`::ffff:192.0.2.128`) and RFC 4007 zone identifiers (`fe80::1%eth0`).
pub fn is_valid_ipv6(candidate: &str) -> bool {
    let s = candidate.split('%').next().unwrap_or(candidate);
    if s.is_empty() {
        return false;
    }

    let (head, embedded_ipv4) = match split_ipv4_tail(s) {
        Some((h, v4)) => (h, Some(v4)),
        None => (s, None),
    };

    if let Some(v4) = embedded_ipv4 {
        if parse_ipv4(v4).is_none() {
            return false;
        }
    }

    // A full address is 8 groups; with an IPv4 tail the last two groups are
    // spent on the four octets, leaving 6.
    let max_groups = if embedded_ipv4.is_some() { 6 } else { 8 };

    match head.split_once("::") {
        Some((left, right)) => {
            if right.contains("::") {
                return false;
            }
            let left = split_groups(left);
            let right = split_groups(right);
            if !left.iter().all(|g| is_hex_group(g)) {
                return false;
            }
            if !right.iter().all(|g| is_hex_group(g)) {
                return false;
            }
            // `::` must stand in for at least one omitted group.
            left.len() + right.len() < max_groups
        }
        None => {
            let groups = split_groups(head);
            groups.len() == max_groups && groups.iter().all(|g| is_hex_group(g))
        }
    }
}

/// Split a trailing IPv4 address off an IPv6 candidate.
///
/// `rsplit_once(':')` is not enough: in `2001:db8::192.0.2.128` the last colon
/// is the *second* half of the `::`, so naively cutting there would leave a
/// dangling `2001:db8:` and lose the compression marker entirely.
fn split_ipv4_tail(s: &str) -> Option<(&str, &str)> {
    let colon = s.rfind(':')?;
    let tail = &s[colon + 1..];

    if !tail.contains('.') || parse_ipv4(tail).is_none() {
        return None;
    }

    // Hand the colon back if we just cut a "::" in half.
    let head_end = if s[..colon].ends_with(':') {
        colon + 1
    } else {
        colon
    };

    Some((&s[..head_end], tail))
}

fn split_groups(s: &str) -> Vec<&str> {
    if s.is_empty() {
        Vec::new()
    } else {
        s.split(':').collect()
    }
}

fn is_hex_group(g: &str) -> bool {
    !g.is_empty() && g.len() <= 4 && g.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luhn_accepts_known_test_cards() {
        assert!(luhn_ok("4111 1111 1111 1111")); // Visa
        assert!(luhn_ok("5500 0000 0000 0004")); // Mastercard
        assert!(luhn_ok("3782 822463 10005")); // Amex, 15 digits
        assert!(luhn_ok("6011111111111117")); // Discover
        assert!(luhn_ok("4222222222222")); // Visa, 13 digits
    }

    #[test]
    fn luhn_rejects_bad_checksum_and_bad_length() {
        assert!(!luhn_ok("1234 5678 9012 3456"));
        assert!(!luhn_ok("4111 1111 1111 1112"));
        assert!(!luhn_ok("411")); // too short
        assert!(!luhn_ok("41111111111111111111")); // too long
    }

    #[test]
    fn card_prefix_filters_noise() {
        assert!(has_card_prefix("4111111111111111"));
        assert!(has_card_prefix("378282246310005"));
        assert!(has_card_prefix("2223003122003222")); // Mastercard 2-series
        assert!(!has_card_prefix("9111111111111111"));
        assert!(!has_card_prefix("1234567890123"));
    }

    #[test]
    fn iban_checksums() {
        assert!(is_valid_iban("DE89 3704 0044 0532 0130 00"));
        assert!(is_valid_iban("GB82 WEST 1234 5698 7654 32"));
        assert!(is_valid_iban("FR1420041010050500013M02606"));
        assert!(is_valid_iban("NL91ABNA0417164300"));

        assert!(!is_valid_iban("DE89 3704 0044 0532 0130 01")); // bad check digits
        assert!(!is_valid_iban("DE89 3704 0044 0532 0130")); // wrong length for DE
        assert!(!is_valid_iban("de89 3704 0044 0532 0130 00")); // lowercase
        assert!(!is_valid_iban("XX89 3704 0044 0532 0130 00")); // not an IBAN shape
    }

    #[test]
    fn ipv6_shapes() {
        assert!(is_valid_ipv6("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        assert!(is_valid_ipv6("2001:db8::ff00:42:8329"));
        assert!(is_valid_ipv6("::1"));
        assert!(is_valid_ipv6("::"));
        assert!(is_valid_ipv6("fe80::1%eth0")); // zone identifier
        assert!(is_valid_ipv6("::ffff:192.0.2.128")); // IPv4-mapped
        assert!(is_valid_ipv6("2001:db8::192.0.2.128")); // compressed + IPv4
        assert!(is_valid_ipv6("2001:db8:0:0:0:0:192.0.2.128")); // full + IPv4
        assert!(is_valid_ipv6("::192.0.2.128")); // bare "::" + IPv4

        assert!(!is_valid_ipv6("1:2:3:4:5:6:7")); // 7 groups and no "::"
        assert!(!is_valid_ipv6("1:2:3:4:5:6:7:8:9")); // too many groups
        assert!(!is_valid_ipv6("1::2::3")); // two "::"
        assert!(!is_valid_ipv6("2001:db8:::1"));
        assert!(!is_valid_ipv6("g001::1")); // non-hex group
        assert!(!is_valid_ipv6("12345::1")); // 5-character group
        assert!(!is_valid_ipv6("::ffff:192.0.2.256")); // bad octet
    }

    #[test]
    fn ipv4_octets() {
        assert_eq!(parse_ipv4("192.168.1.1"), Some([192, 168, 1, 1]));
        assert_eq!(parse_ipv4("0.0.0.0"), Some([0, 0, 0, 0]));
        assert_eq!(parse_ipv4("255.255.255.255"), Some([255, 255, 255, 255]));

        assert_eq!(parse_ipv4("256.1.1.1"), None); // out of range
        assert_eq!(parse_ipv4("192.168.1"), None); // three octets
        assert_eq!(parse_ipv4("192.168.1.1.1"), None); // five octets
        assert_eq!(parse_ipv4("192.168.01.1"), None); // leading zero
        assert_eq!(parse_ipv4("192.168.1."), None); // trailing dot
    }
}
