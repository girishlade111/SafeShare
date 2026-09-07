//! pii_engine — offline PII detection.
//!
//! Regex-based detectors for email, phone, IPv4/IPv6, payment cards, IBAN and
//! person names. Candidates are then checked against the real checksum or
//! structural rules for their type, which is what keeps timestamps, version
//! strings and arbitrary digit runs out of the results.
//!
//! No network access, no filesystem access, no ambient state.

use serde::{Deserialize, Serialize};

mod detectors;
mod validate;

#[cfg(test)]
mod detection_tests;

/// Kinds of PII the engine can detect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PiiType {
    Email,
    Phone,
    Ipv4,
    Ipv6,
    CreditCard,
    Iban,
    Name,
}

impl PiiType {
    /// Stable string form, matching the serde representation.
    pub fn as_str(self) -> &'static str {
        match self {
            PiiType::Email => "email",
            PiiType::Phone => "phone",
            PiiType::Ipv4 => "ipv4",
            PiiType::Ipv6 => "ipv6",
            PiiType::CreditCard => "credit_card",
            PiiType::Iban => "iban",
            PiiType::Name => "name",
        }
    }
}

/// A single PII hit inside the scanned text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PiiMatch {
    /// What kind of PII was found.
    pub r#type: PiiType,
    /// The exact slice of input that matched.
    pub matched_text: String,
    /// Offset of the first character of the match.
    pub start_index: usize,
    /// Offset one past the last character of the match.
    pub end_index: usize,
    /// 0.0–1.0. Checksum-validated types score high; the heuristic name
    /// detector is deliberately low so callers can gate on it.
    pub confidence: f32,
}

/// Scan `text` and return every PII hit.
///
/// Results are ordered by position and never overlap: when two detectors claim
/// the same characters the more trustworthy one wins.
///
/// Offsets are **character** indices, not byte indices, so JavaScript callers
/// can `text.slice(startIndex, endIndex)` directly. For ASCII input the two
/// are identical.
pub fn detect(text: &str) -> Vec<PiiMatch> {
    let mut candidates = Vec::new();
    detectors::emails(text, &mut candidates);
    detectors::phones(text, &mut candidates);
    detectors::ipv4s(text, &mut candidates);
    detectors::ipv6s(text, &mut candidates);
    detectors::credit_cards(text, &mut candidates);
    detectors::ibans(text, &mut candidates);
    detectors::names(text, &mut candidates);

    // Strongest claim first; within a tie, leftmost then longest.
    candidates.sort_by(|a, b| {
        detectors::priority(b.r#type)
            .cmp(&detectors::priority(a.r#type))
            .then_with(|| a.start_index.cmp(&b.start_index))
            .then_with(|| (b.end_index - b.start_index).cmp(&(a.end_index - a.start_index)))
    });

    // Greedily keep whatever does not collide with something already kept.
    let mut kept: Vec<PiiMatch> = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let collides = kept
            .iter()
            .any(|k| candidate.start_index < k.end_index && k.start_index < candidate.end_index);
        if !collides {
            kept.push(candidate);
        }
    }

    kept.sort_by_key(|m| m.start_index);

    if text.is_ascii() {
        kept // byte offsets already equal char offsets
    } else {
        for m in &mut kept {
            m.start_index = to_char_offset(text, m.start_index);
            m.end_index = to_char_offset(text, m.end_index);
        }
        kept
    }
}

fn to_char_offset(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset].chars().count()
}

/// Metadata about the engine itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: &'static str,
    pub version: &'static str,
}

pub fn engine_info() -> EngineInfo {
    EngineInfo {
        name: "pii_engine",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_version() {
        let info = engine_info();
        assert_eq!(info.name, "pii_engine");
        assert!(!info.version.is_empty());
    }

    #[test]
    fn empty_input_yields_nothing() {
        assert!(detect("").is_empty());
        assert!(detect("nothing to see here").is_empty());
    }

    #[test]
    fn offsets_are_character_indices() {
        // "Ü" is two bytes, so byte offsets would be shifted by one.
        let text = "Kontakt: Ü. Müller <u.mueller@example.org>";
        let matches = detect(text);

        let email = matches
            .iter()
            .find(|m| m.r#type == PiiType::Email)
            .expect("email should be detected");

        assert_eq!(
            slice_chars(text, email.start_index, email.end_index),
            "u.mueller@example.org"
        );
    }

    fn slice_chars(text: &str, start: usize, end: usize) -> String {
        text.chars().skip(start).take(end - start).collect()
    }
}
