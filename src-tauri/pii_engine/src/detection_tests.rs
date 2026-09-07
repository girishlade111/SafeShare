//! End-to-end tests against realistically messy log text.
//!
//! These exercise the public `detect()` API only — the same surface the Tauri
//! command exposes — so they keep passing across internal refactors.

use crate::{detect, PiiMatch, PiiType};

/// Deliberately unpleasant input: mixed PII types, PII embedded in URLs and
/// JSON, and near-misses that must *not* be reported.
const MESSY_LOG: &str = r#"
2026-08-30T12:34:56.789Z INFO  request received id=7f3c9e1a
  client_ip="192.168.13.42" via 10.0.0.7 -> upstream 8.8.8.8
  {"user":"alice","email":"alice.nguyen@example.com","ip":"172.16.254.1","ua":"curl/8.4.0"}
  contact: bob.smith+newsletter@sub.domain.co.uk, cc: carol@corp.io
  mailto:dana.lee@partner.org?subject=Invoice%20#4417
  portal=https://app.example.com/users/verify?email=erin.dubois@corp.io&token=abc123
  asset=https://cdn.example.com/img/a@b.png (not an email)
  phone_primary="+1 415 555 2671" phone_alt="(415) 555-2672" intl="+44 20 7946 0958"
  fax=+49 30 901820 mobile=+919876543210 e164=+14155552673
  payment card 4111 1111 1111 1111 exp 12/29 cvv 123
  amex 3782 822463 10005
  invalid card 1234 5678 9012 3456 (fails Luhn)
  iban DE89 3704 0044 0532 0130 00 and GB82 WEST 1234 5698 7654 32
  bad iban DE89 3704 0044 0532 0130 01
  customer John Smith visited, assisted by Maria Garcia
  ipv6 full 2001:0db8:85a3:0000:0000:8a2e:0370:7334
  ipv6 short 2001:db8::ff00:42:8329 and loopback ::1
  ipv6 mapped ::ffff:192.0.2.128
  version 10.20.30 build 4417 date 2026-08-30 duration 12:34
"#;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Offsets are character indices, so slice by chars rather than bytes.
fn slice_chars(text: &str, start: usize, end: usize) -> String {
    text.chars().skip(start).take(end - start).collect()
}

fn found<'a>(matches: &'a [PiiMatch], needle: &str) -> Option<&'a PiiMatch> {
    matches.iter().find(|m| m.matched_text == needle)
}

fn assert_detected(text: &str, matches: &[PiiMatch], needle: &str, expected: PiiType) {
    let m = found(matches, needle).unwrap_or_else(|| {
        panic!(
            "expected {needle:?} to be detected as {expected:?}\n\nmatches found:\n{}",
            render(matches)
        )
    });

    assert_eq!(m.r#type, expected, "wrong type for {needle:?}");
    assert_eq!(
        slice_chars(text, m.start_index, m.end_index),
        needle,
        "offsets [{}, {}) do not slice back to {needle:?}",
        m.start_index,
        m.end_index
    );
}

fn assert_not_detected(matches: &[PiiMatch], needle: &str) {
    assert!(
        found(matches, needle).is_none(),
        "expected {needle:?} NOT to be detected\n\nmatches found:\n{}",
        render(matches)
    );
}

fn count_of(matches: &[PiiMatch], kind: PiiType) -> usize {
    matches.iter().filter(|m| m.r#type == kind).count()
}

fn render(matches: &[PiiMatch]) -> String {
    matches
        .iter()
        .map(|m| {
            format!(
                "  {:?} {:?} @ [{}, {})",
                m.r#type, m.matched_text, m.start_index, m.end_index
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[test]
fn finds_one_of_every_type() {
    let matches = detect(MESSY_LOG);

    assert_detected(MESSY_LOG, &matches, "alice.nguyen@example.com", PiiType::Email);
    assert_detected(MESSY_LOG, &matches, "+1 415 555 2671", PiiType::Phone);
    assert_detected(MESSY_LOG, &matches, "192.168.13.42", PiiType::Ipv4);
    assert_detected(MESSY_LOG, &matches, "2001:db8::ff00:42:8329", PiiType::Ipv6);
    assert_detected(MESSY_LOG, &matches, "4111 1111 1111 1111", PiiType::CreditCard);
    assert_detected(MESSY_LOG, &matches, "DE89 3704 0044 0532 0130 00", PiiType::Iban);
    assert_detected(MESSY_LOG, &matches, "John Smith", PiiType::Name);
}

#[test]
fn finds_every_expected_match() {
    let m = detect(MESSY_LOG);

    // Emails: five real ones; `a@b.png` is a URL path and is excluded.
    assert_detected(MESSY_LOG, &m, "alice.nguyen@example.com", PiiType::Email);
    assert_detected(MESSY_LOG, &m, "bob.smith+newsletter@sub.domain.co.uk", PiiType::Email);
    assert_detected(MESSY_LOG, &m, "carol@corp.io", PiiType::Email);
    assert_detected(MESSY_LOG, &m, "dana.lee@partner.org", PiiType::Email);
    assert_detected(MESSY_LOG, &m, "erin.dubois@corp.io", PiiType::Email);
    assert_eq!(count_of(&m, PiiType::Email), 5);

    // Phones across four formatting conventions.
    assert_detected(MESSY_LOG, &m, "+1 415 555 2671", PiiType::Phone); // intl, spaced
    assert_detected(MESSY_LOG, &m, "(415) 555-2672", PiiType::Phone); // national, parens
    assert_detected(MESSY_LOG, &m, "+44 20 7946 0958", PiiType::Phone); // UK
    assert_detected(MESSY_LOG, &m, "+49 30 901820", PiiType::Phone); // Germany
    assert_detected(MESSY_LOG, &m, "+919876543210", PiiType::Phone); // E.164, unseparated
    assert_detected(MESSY_LOG, &m, "+14155552673", PiiType::Phone); // E.164, unseparated
    assert_eq!(count_of(&m, PiiType::Phone), 6);

    // IPs: four IPv4 and four IPv6 (see the nesting test below).
    assert_detected(MESSY_LOG, &m, "192.168.13.42", PiiType::Ipv4);
    assert_detected(MESSY_LOG, &m, "10.0.0.7", PiiType::Ipv4);
    assert_detected(MESSY_LOG, &m, "8.8.8.8", PiiType::Ipv4);
    assert_detected(MESSY_LOG, &m, "172.16.254.1", PiiType::Ipv4);
    assert_eq!(count_of(&m, PiiType::Ipv4), 4);

    assert_detected(MESSY_LOG, &m, "2001:0db8:85a3:0000:0000:8a2e:0370:7334", PiiType::Ipv6);
    assert_detected(MESSY_LOG, &m, "2001:db8::ff00:42:8329", PiiType::Ipv6);
    assert_detected(MESSY_LOG, &m, "::1", PiiType::Ipv6);
    assert_detected(MESSY_LOG, &m, "::ffff:192.0.2.128", PiiType::Ipv6);
    assert_eq!(count_of(&m, PiiType::Ipv6), 4);

    // Cards and IBANs: only the checksum-valid ones.
    assert_detected(MESSY_LOG, &m, "4111 1111 1111 1111", PiiType::CreditCard);
    assert_detected(MESSY_LOG, &m, "3782 822463 10005", PiiType::CreditCard);
    assert_eq!(count_of(&m, PiiType::CreditCard), 2);

    assert_detected(MESSY_LOG, &m, "DE89 3704 0044 0532 0130 00", PiiType::Iban);
    assert_detected(MESSY_LOG, &m, "GB82 WEST 1234 5698 7654 32", PiiType::Iban);
    assert_eq!(count_of(&m, PiiType::Iban), 2);

    // Names.
    assert_detected(MESSY_LOG, &m, "John Smith", PiiType::Name);
    assert_detected(MESSY_LOG, &m, "Maria Garcia", PiiType::Name);
    assert_eq!(count_of(&m, PiiType::Name), 2);

    assert_eq!(m.len(), 25, "unexpected total match count");
}

#[test]
fn email_inside_url_query_is_detected() {
    let text = "GET https://app.example.com/u/verify?email=erin.dubois@corp.io&t=1";
    let matches = detect(text);

    assert_detected(text, &matches, "erin.dubois@corp.io", PiiType::Email);
}

#[test]
fn email_inside_mailto_is_detected() {
    let text = "reach out via mailto:dana.lee@partner.org?subject=Hi there";
    let matches = detect(text);

    assert_detected(text, &matches, "dana.lee@partner.org", PiiType::Email);
}

#[test]
fn email_shaped_url_path_is_ignored() {
    // `a@b.png` sits after a path separator and ends in a file extension.
    let matches = detect("asset=https://cdn.example.com/img/a@b.png (not an email)");

    assert_not_detected(&matches, "a@b.png");
    assert!(
        matches.is_empty(),
        "expected no matches, got:\n{}",
        render(&matches)
    );
}

#[test]
fn ip_inside_json_is_detected() {
    let text = r#"{"clientIp":"172.16.254.1","port":8080,"ua":"curl/8.4.0"}"#;
    let matches = detect(text);

    assert_detected(text, &matches, "172.16.254.1", PiiType::Ipv4);
}

#[test]
fn ipv6_wins_over_the_ipv4_nested_in_it() {
    let text = "mapped ::ffff:192.0.2.128 done";
    let matches = detect(text);

    assert_detected(text, &matches, "::ffff:192.0.2.128", PiiType::Ipv6);
    assert_not_detected(&matches, "192.0.2.128");
    assert_eq!(matches.len(), 1);
}

#[test]
fn luhn_invalid_card_is_ignored() {
    let matches = detect("invalid card 1234 5678 9012 3456 (fails Luhn)");

    assert_not_detected(&matches, "1234 5678 9012 3456");
}

#[test]
fn card_with_bad_iban_checksum_is_ignored() {
    let matches = detect("bad iban DE89 3704 0044 0532 0130 01");

    assert_not_detected(&matches, "DE89 3704 0044 0532 0130 01");
}

#[test]
fn timestamps_and_versions_are_not_phones() {
    let matches = detect("2026-08-30T12:34:56.789Z version 10.20.30 build 4417 duration 12:34");

    assert!(
        matches.is_empty(),
        "expected no matches, got:\n{}",
        render(&matches)
    );
}

#[test]
fn short_digit_runs_are_not_phones() {
    let matches = detect("cvv 123 exp 12/29 port 8080 qty 42");

    assert!(
        matches.is_empty(),
        "expected no matches, got:\n{}",
        render(&matches)
    );
}

#[test]
fn sentence_case_prose_is_not_a_name() {
    let matches = detect("Request failed. Payment declined. Retry scheduled.");

    assert!(
        matches.iter().all(|m| m.r#type != PiiType::Name),
        "log prose should not yield names, got:\n{}",
        render(&matches)
    );
}

#[test]
fn matches_never_overlap_and_are_ordered() {
    let matches = detect(MESSY_LOG);

    for window in matches.windows(2) {
        assert!(
            window[0].end_index <= window[1].start_index,
            "matches overlap or are out of order: {:?} then {:?}",
            window[0].matched_text,
            window[1].matched_text
        );
    }
}

#[test]
fn every_offset_slices_back_to_its_text() {
    let matches = detect(MESSY_LOG);

    for m in &matches {
        assert_eq!(
            slice_chars(MESSY_LOG, m.start_index, m.end_index),
            m.matched_text,
            "offsets [{}, {}) do not reproduce {:?}",
            m.start_index,
            m.end_index,
            m.matched_text
        );
    }
}

#[test]
fn name_detector_is_low_confidence() {
    let matches = detect("customer John Smith visited");

    let name = matches
        .iter()
        .find(|m| m.r#type == PiiType::Name)
        .expect("John Smith should be detected");
    assert!(
        name.confidence < 0.5,
        "heuristic names should score low, got {}",
        name.confidence
    );
}

#[test]
fn checksum_validated_types_are_high_confidence() {
    let matches = detect("card 4111 1111 1111 1111 iban DE89 3704 0044 0532 0130 00");

    assert_eq!(matches.len(), 2);
    for m in &matches {
        assert!(
            m.confidence >= 0.9,
            "{:?} should be high confidence, got {}",
            m.r#type,
            m.confidence
        );
    }
}

#[test]
fn detection_is_deterministic() {
    let a = detect(MESSY_LOG);
    let b = detect(MESSY_LOG);
    assert_eq!(a, b);
}
