//! Regex-driven candidate generation, one function per PII type.
//!
//! Every regex here is deliberately *loose*: it over-produces, and the real
//! rules in [`crate::validate`] decide what survives. Keeping the two apart
//! makes each detector easy to reason about and easy to test.
//!
//! Note that the `regex` crate has no lookaround, so boundary checks that
//! would normally be `(?<!…)` are done in Rust below.

use std::ops::Range;
use std::sync::LazyLock;

use regex::{Match, Regex};

use crate::validate;
use crate::{PiiMatch, PiiType};

/// Relative trust in each detector.
///
/// When two candidates overlap the higher number wins: a Luhn-valid card
/// outranks the phone-shaped digit run inside it, and an IPv6 outranks the
/// IPv4 address nested in its tail.
pub fn priority(t: PiiType) -> u8 {
    match t {
        PiiType::Iban => 90,
        PiiType::CreditCard => 90,
        PiiType::Email => 85,
        PiiType::Ipv6 => 80,
        PiiType::Ipv4 => 70,
        PiiType::Phone => 50,
        PiiType::Name => 10,
    }
}

// ---------------------------------------------------------------------------
// Email
// ---------------------------------------------------------------------------

static EMAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9](?:[A-Za-z0-9\-]*[A-Za-z0-9])?(?:\.[A-Za-z0-9](?:[A-Za-z0-9\-]*[A-Za-z0-9])?)*\.[A-Za-z]{2,24}",
    )
    .expect("email regex must compile")
});

/// File extensions that make an `@` inside a URL path look like an address.
const URL_ASSET_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "svg", "webp", "ico", "css", "js", "mjs", "ts", "tsx", "jsx",
    "json", "html", "htm", "xml", "pdf", "zip", "gz", "tar", "woff", "woff2", "ttf", "eot",
    "mp4", "webm", "map", "txt", "md", "csv",
];

pub fn emails(text: &str, out: &mut Vec<PiiMatch>) {
    for m in EMAIL.find_iter(text) {
        // `https://cdn.example.com/img/a@b.png` is a path, not an address.
        if is_url_asset(text, &m) {
            continue;
        }
        push(out, &m, PiiType::Email, 0.95);
    }
}

fn is_url_asset(text: &str, m: &Match) -> bool {
    if char_before(text, m.start()) != Some('/') {
        return false;
    }
    let tld = m.as_str().rsplit('.').next().unwrap_or_default();
    URL_ASSET_EXT.contains(&tld)
}

// ---------------------------------------------------------------------------
// Phone
// ---------------------------------------------------------------------------

static PHONE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?:",
        r"\+\d{7,15}",                                    // E.164, unseparated
        r"|",
        r"\+\d{1,3}[ .\-]?(?:\(\d{1,5}\)[ .\-]?|\d{2,5}[ .\-])\d{2,5}(?:[ .\-]?\d{2,5}){1,3}", // +CC …
        r"|",
        r"(?:\(\d{1,5}\)[ .\-]?|\d{2,5}[ .\-])\d{2,5}(?:[ .\-]?\d{2,5}){1,3}",                 // national
        r")",
    ))
    .expect("phone regex must compile")
});

/// Leading `YYYY-MM-DD` / `DD-MM-YYYY`. Log timestamps are the single biggest
/// source of phone false positives, so they get an explicit blocklist.
static DATE_LEAD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:19|20)\d{2}[-/.]\d{1,2}[-/.]\d{1,2}")
        .expect("date-lead regex must compile")
});

pub fn phones(text: &str, out: &mut Vec<PiiMatch>) {
    for m in PHONE.find_iter(text) {
        let raw = m.as_str();

        // Never slice a longer token in half.
        if char_before(text, m.start()).is_some_and(|c| c.is_ascii_digit())
            || char_after(text, m.end()).is_some_and(|c| c.is_ascii_digit())
        {
            continue;
        }

        // `2026-08-30T12:34:56` is a timestamp, not a number you can dial.
        if DATE_LEAD.is_match(raw) {
            continue;
        }

        let digit_count = raw.bytes().filter(|b| b.is_ascii_digit()).count();
        if digit_count > 15 {
            continue; // E.164 ceiling; also keeps 16-digit cards out
        }

        // A bare run of digits is only convincing at national-number length.
        // Anything shorter needs a `+` or parentheses to be believed.
        let looks_deliberate =
            raw.starts_with('+') || raw.contains('(') || raw.contains(')') || digit_count >= 9;
        if !looks_deliberate {
            continue;
        }

        push(out, &m, PiiType::Phone, 0.75);
    }
}

// ---------------------------------------------------------------------------
// IPv4
// ---------------------------------------------------------------------------

static IPV4: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"\b(?:(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.){3}",
        r"(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\b",
    ))
    .expect("ipv4 regex must compile")
});

pub fn ipv4s(text: &str, out: &mut Vec<PiiMatch>) {
    for m in IPV4.find_iter(text) {
        // The regex already constrains octets; this catches nothing extra but
        // keeps the validation path identical for every detector.
        if validate::parse_ipv4(m.as_str()).is_none() {
            continue;
        }
        push(out, &m, PiiType::Ipv4, 0.95);
    }
}

// ---------------------------------------------------------------------------
// IPv6
// ---------------------------------------------------------------------------

static IPV6: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        // Eight explicit groups.
        r"[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4}){7}",
        "|",
        // Compressed, with an embedded IPv4 tail.
        r"[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4}){0,5}::(?:[0-9A-Fa-f]{1,4}:){0,4}(?:[0-9]{1,3}\.){3}[0-9]{1,3}",
        "|",
        // Compressed.
        r"[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4}){0,6}::(?:[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4}){0,5})?",
        "|",
        // Leading "::" with an embedded IPv4 tail.
        r"::(?:[0-9A-Fa-f]{1,4}:){0,4}(?:[0-9]{1,3}\.){3}[0-9]{1,3}",
        "|",
        // Leading "::".
        r"::(?:[0-9A-Fa-f]{1,4}(?::[0-9A-Fa-f]{1,4}){0,6})?",
    ))
    .expect("ipv6 regex must compile")
});

pub fn ipv6s(text: &str, out: &mut Vec<PiiMatch>) {
    for m in IPV6.find_iter(text) {
        // Without lookaround the regex can start or end mid-token; require a
        // clean break on both sides.
        let clean_before = !char_before(text, m.start())
            .is_some_and(|c| c.is_ascii_hexdigit() || c == ':');
        let clean_after =
            !char_after(text, m.end()).is_some_and(|c| c.is_ascii_hexdigit() || c == ':' || c == '.');

        if !clean_before || !clean_after || !validate::is_valid_ipv6(m.as_str()) {
            continue;
        }
        push(out, &m, PiiType::Ipv6, 0.9);
    }
}

// ---------------------------------------------------------------------------
// Credit card
// ---------------------------------------------------------------------------

static CREDIT_CARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d[ \-]?){12,18}\d\b").expect("card regex must compile")
});

pub fn credit_cards(text: &str, out: &mut Vec<PiiMatch>) {
    for m in CREDIT_CARD.find_iter(text) {
        // Reject candidates carved out of a longer numeric run.
        if !is_standalone_number(text, m.start(), m.end()) {
            continue;
        }
        if !validate::luhn_ok(m.as_str()) || !validate::has_card_prefix(m.as_str()) {
            continue;
        }
        push(out, &m, PiiType::CreditCard, 0.95);
    }
}

/// True when the match is not merely a slice of a longer digit-and-separator
/// run (`…1234-5678-9012-3456-7890…`).
fn is_standalone_number(text: &str, start: usize, end: usize) -> bool {
    let bytes = text.as_bytes();

    if start > 0 && bytes[start - 1].is_ascii_digit() {
        return false;
    }
    if end < bytes.len() && bytes[end].is_ascii_digit() {
        return false;
    }

    // A separator hugging a digit on either side means there is more number
    // where this one came from.
    let glued_left = start > 1
        && (bytes[start - 1] == b'-' || bytes[start - 1] == b' ')
        && bytes[start - 2].is_ascii_digit();
    let glued_right = end + 1 < bytes.len()
        && (bytes[end] == b'-' || bytes[end] == b' ')
        && bytes[end + 1].is_ascii_digit();

    !glued_left && !glued_right
}

// ---------------------------------------------------------------------------
// IBAN
// ---------------------------------------------------------------------------

static IBAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Z]{2}\d{2}(?:[ ]?[A-Z0-9]{4}){2,7}(?:[ ]?[A-Z0-9]{1,4})?\b")
        .expect("iban regex must compile")
});

pub fn ibans(text: &str, out: &mut Vec<PiiMatch>) {
    for m in IBAN.find_iter(text) {
        if !validate::is_valid_iban(m.as_str()) {
            continue;
        }
        push(out, &m, PiiType::Iban, 0.95);
    }
}

// ---------------------------------------------------------------------------
// Name (heuristic)
// ---------------------------------------------------------------------------

static NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b[A-Z][a-z]{1,20}(?:-[A-Z][a-z]{1,20})?[ \t]+[A-Z][a-z]{1,20}(?:-[A-Z][a-z]{1,20})?\b",
    )
    .expect("name regex must compile")
});

/// Words that routinely open a sentence or name a log field. Checked against
/// *both* tokens of a candidate pair, since "Payment Failed" is no more a
/// person than "Failed Payment" is.
const NAME_STOPWORDS: &[&str] = &[
    // log / prose noise
    "the", "this", "that", "these", "those", "there", "their", "then", "they", "its",
    "error", "errors", "warn", "warning", "info", "debug", "trace", "fatal", "notice", "log",
    "failed", "failure", "success", "unable", "cannot", "could", "should", "would",
    "invalid", "missing", "unknown", "unexpected", "exception", "timeout", "timed", "panic",
    "request", "response", "server", "client", "connection", "database", "service", "worker",
    "user", "users", "session", "sessions", "token", "tokens", "password", "login", "logout",
    "auth", "admin", "root", "guest", "system", "process", "thread", "task", "job", "event",
    "please", "note", "notes", "from", "with", "without", "using", "used",
    "processing", "starting", "stopping", "loading", "saving", "updating", "deleting",
    "creating", "created", "updated", "deleted", "received", "sent", "opening", "closing",
    "retrying", "retry", "skipping", "skipped", "waiting", "running", "finished",
    // protocols and formats
    "http", "https", "json", "xml", "html", "api", "url", "uri", "uuid", "get", "post",
    "put", "delete", "patch", "head", "options", "content", "type", "accept",
    // calendar
    "january", "february", "march", "april", "may", "june", "july", "august", "september",
    "october", "november", "december", "jan", "feb", "mar", "apr", "jun", "jul", "aug",
    "sep", "sept", "oct", "nov", "dec",
    "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
    "mon", "tue", "tues", "wed", "thu", "thur", "thurs", "fri", "sat", "sun",
    // quantifiers and generic adjectives
    "new", "old", "first", "last", "next", "previous", "total", "average", "max", "min",
    "all", "any", "some", "none", "true", "false", "null", "ok", "okay",
    // domain vocabulary that sits next to real PII
    "card", "credit", "debit", "visa", "mastercard", "amex", "iban", "bic", "swift",
    "email", "mail", "phone", "mobile", "fax", "address", "ipv4", "ipv6", "mac",
    "network", "router", "gateway", "dns", "tcp", "udp", "ssl", "tls", "ssh",
    // structural
    "file", "line", "code", "name", "value", "key", "data", "version", "build",
    "date", "time", "duration", "status", "state", "level", "message", "source", "target",
    "customer", "account", "invoice", "payment", "order", "product", "company", "support",
];

pub fn names(text: &str, out: &mut Vec<PiiMatch>) {
    for m in NAME.find_iter(text) {
        let is_noise = m
            .as_str()
            .split_whitespace()
            .any(|word| NAME_STOPWORDS.contains(&word.to_ascii_lowercase().as_str()));

        if is_noise {
            continue;
        }
        push(out, &m, PiiType::Name, 0.3);
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn push(out: &mut Vec<PiiMatch>, m: &Match, kind: PiiType, confidence: f32) {
    let Range { start, end } = m.range();
    out.push(PiiMatch {
        r#type: kind,
        matched_text: m.as_str().to_string(),
        start_index: start,
        end_index: end,
        confidence,
    });
}

fn char_before(text: &str, byte_index: usize) -> Option<char> {
    text[..byte_index].chars().next_back()
}

fn char_after(text: &str, byte_index: usize) -> Option<char> {
    text[byte_index..].chars().next()
}
