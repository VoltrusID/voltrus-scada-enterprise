//! 21 CFR Part 11 exports as PDF: the electronic signature records and the audit
//! trail, printed with the PDF writer of the scheduled reports.
//!
//! Every signature shows its manifestation (§11.50): the signer's printed name and
//! user name, the date and time of signing with its time zone, the meaning and the
//! reason, plus the record checksum and whether it still verifies. Every page has
//! the export's filter in its header and "page n of m" in its footer.

use time::{OffsetDateTime, UtcOffset};
use voltrus::compliance::ElectronicSignature;

use crate::reports::render::pdf::{Font, MARGIN, Writer, wrap};

/// What the header and the first page say about an export.
pub struct ExportMeta {
    pub title: String,
    /// Brand title of the installation (or "Voltrus SCADA").
    pub site: String,
    /// The filter, e.g. "2026-11-01 00:00 to 2026-11-16 23:59 UTC+07:00, user alice".
    pub filter: String,
    pub generated_at: i64,
    pub generated_by: String,
    /// Minutes east of UTC the times are printed in.
    pub offset_minutes: i32,
}

pub struct SignatureEntry {
    pub sig: ElectronicSignature,
    /// The signer's full name as recorded when they signed, or the current one.
    pub full_name: Option<String>,
    /// The name was not recorded at signing; it is the one on file now.
    pub name_is_current: bool,
    pub hash_valid: bool,
}

pub struct AuditEntry {
    pub id: i64,
    pub created_at: i64,
    pub username: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
    /// The row's link in the audit hash chain (absent on rows from before the chain).
    pub hash: Option<String>,
}

/// The result of walking the whole audit hash chain.
pub struct ChainSummary {
    pub ok: bool,
    pub checked: u64,
    pub unhashed_legacy: u64,
    pub first_bad_id: Option<i64>,
}

fn offset(minutes: i32) -> UtcOffset {
    UtcOffset::from_whole_seconds(minutes * 60).unwrap_or(UtcOffset::UTC)
}

/// `UTC+07:00`, `UTC-03:30`, `UTC`.
pub fn zone_label(minutes: i32) -> String {
    if minutes == 0 {
        return "UTC".into();
    }
    let sign = if minutes < 0 { '-' } else { '+' };
    let m = minutes.unsigned_abs();
    format!("UTC{sign}{:02}:{:02}", m / 60, m % 60)
}

/// `2026-11-16 14:03:22 UTC+07:00`.
pub fn format_time(ts: i64, minutes: i32) -> String {
    let Ok(t) = OffsetDateTime::from_unix_timestamp(ts) else {
        return ts.to_string();
    };
    let t = t.to_offset(offset(minutes));
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} {}",
        t.year(),
        u8::from(t.month()),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
        zone_label(minutes)
    )
}

fn cover(w: &mut Writer, meta: &ExportMeta, lines: &[String]) {
    w.text_in(Font::Bold, MARGIN, w.y - 6.0, 18.0, &meta.title, 60);
    w.y -= 26.0;
    w.text_in(Font::Regular, MARGIN, w.y, 10.0, &meta.site, 90);
    w.y -= 14.0;
    for line in [
        format!("Filter: {}", meta.filter),
        format!(
            "Generated {} by {}",
            format_time(meta.generated_at, meta.offset_minutes),
            meta.generated_by
        ),
    ]
    .iter()
    .chain(lines)
    {
        for part in wrap(line, 100) {
            w.text_in(Font::Regular, MARGIN, w.y, 9.0, &part, 110);
            w.y -= 12.0;
        }
    }
    w.y -= 4.0;
    w.rule(w.y, 0.6);
    w.y -= 16.0;
}

/// Print `label` and the wrapped `value` at the current line; returns the lines used.
fn field(w: &mut Writer, label: &str, value: &str, font: Font, width: usize) {
    let lines = wrap(value, width);
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            w.text_in(Font::Bold, MARGIN + 12.0, w.y, 8.5, label, 20);
        }
        w.text_in(font, MARGIN + 92.0, w.y, 8.5, line, width + 2);
        w.y -= 11.0;
    }
}

fn field_lines(value: &str, width: usize) -> usize {
    wrap(value, width).len()
}

/// The signature records as a PDF document.
pub fn signatures_pdf(meta: &ExportMeta, entries: &[SignatureEntry]) -> Vec<u8> {
    let mut w = Writer::with_header(format!("{} - {} - {}", meta.site, meta.title, meta.filter));
    let invalid = entries.iter().filter(|e| !e.hash_valid).count();
    cover(
        &mut w,
        meta,
        &[format!(
            "{} signature record(s); {} with a checksum that does not verify",
            entries.len(),
            invalid
        )],
    );
    if entries.is_empty() {
        w.text_in(
            Font::Regular,
            MARGIN,
            w.y,
            10.0,
            "No electronic signatures match the filter.",
            100,
        );
    }
    const WIDTH: usize = 82;
    for e in entries {
        let s = &e.sig;
        let signer = match (&e.full_name, e.name_is_current) {
            (Some(name), false) => format!("{name} ({})", s.user_id),
            (Some(name), true) => format!(
                "{name} ({}) - name on file at export; not recorded at signing",
                s.user_id
            ),
            (None, _) => format!("{} (no full name on file)", s.user_id),
        };
        let record = format!("{} / {}", s.resource_type, s.resource_id);
        let checksum = format!(
            "{}  {}",
            s.signature_hash,
            if e.hash_valid {
                "verified"
            } else {
                "DOES NOT VERIFY"
            }
        );
        let height = 16.0
            + 11.0
                * (field_lines(&signer, WIDTH)
                    + 1
                    + field_lines(&s.meaning, WIDTH)
                    + field_lines(&s.reason, WIDTH)
                    + field_lines(&record, WIDTH)
                    + field_lines(&checksum, 66)) as f64
            + 10.0;
        w.need(height);
        w.text_in(
            Font::Bold,
            MARGIN,
            w.y,
            10.0,
            &format!("Signature #{}", s.id),
            40,
        );
        w.y -= 14.0;
        field(&mut w, "Signed by", &signer, Font::Regular, WIDTH);
        field(
            &mut w,
            "Date and time",
            &format_time(s.created_at, meta.offset_minutes),
            Font::Regular,
            WIDTH,
        );
        field(&mut w, "Meaning", &s.meaning, Font::Regular, WIDTH);
        field(&mut w, "Reason", &s.reason, Font::Regular, WIDTH);
        field(&mut w, "Record", &record, Font::Regular, WIDTH);
        field(&mut w, "Checksum", &checksum, Font::Mono, 66);
        w.y -= 3.0;
        w.rule(w.y + 4.0, 0.85);
        w.y -= 7.0;
    }
    w.finish(&format!("{} - {}", meta.site, meta.title))
}

/// The audit trail as a PDF document.
pub fn audit_pdf(meta: &ExportMeta, entries: &[AuditEntry], chain: &ChainSummary) -> Vec<u8> {
    let mut w = Writer::with_header(format!("{} - {} - {}", meta.site, meta.title, meta.filter));
    let chain_line = if chain.ok {
        format!(
            "Hash chain of the whole audit trail verified: {} entries intact{}",
            chain.checked,
            if chain.unhashed_legacy > 0 {
                format!(
                    ", {} older entries from before the chain",
                    chain.unhashed_legacy
                )
            } else {
                String::new()
            }
        )
    } else {
        format!(
            "Hash chain BROKEN at entry {}: an entry was changed, inserted or deleted",
            chain.first_bad_id.unwrap_or_default()
        )
    };
    cover(
        &mut w,
        meta,
        &[format!("{} entries", entries.len()), chain_line],
    );
    if entries.is_empty() {
        w.text_in(
            Font::Regular,
            MARGIN,
            w.y,
            10.0,
            "No audit entries match the filter.",
            100,
        );
    }
    const WIDTH: usize = 82;
    for e in entries {
        let details = e.details.clone().unwrap_or_default();
        let hash = e
            .hash
            .clone()
            .unwrap_or_else(|| "(before the hash chain)".into());
        let height = 14.0
            + 11.0
                * (1 + field_lines(&e.resource, WIDTH)
                    + if details.is_empty() {
                        0
                    } else {
                        field_lines(&details, WIDTH)
                    }
                    + 1) as f64
            + 8.0;
        w.need(height);
        w.text_in(
            Font::Bold,
            MARGIN,
            w.y,
            9.0,
            &format!(
                "#{}  {}  {}",
                e.id,
                format_time(e.created_at, meta.offset_minutes),
                e.action
            ),
            100,
        );
        w.y -= 12.0;
        field(&mut w, "User", &e.username, Font::Regular, WIDTH);
        field(&mut w, "Resource", &e.resource, Font::Regular, WIDTH);
        if !details.is_empty() {
            field(&mut w, "Details", &details, Font::Regular, WIDTH);
        }
        field(&mut w, "Checksum", &hash, Font::Mono, 66);
        w.y -= 2.0;
        w.rule(w.y + 4.0, 0.85);
        w.y -= 6.0;
    }
    w.finish(&format!("{} - {}", meta.site, meta.title))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta() -> ExportMeta {
        ExportMeta {
            title: "Electronic signature records".into(),
            site: "Plant (A)".into(),
            filter: "2026-11-01 00:00 to 2026-11-16 23:59 UTC+07:00".into(),
            generated_at: 1_794_830_000,
            generated_by: "qa.lead".into(),
            offset_minutes: 420,
        }
    }

    fn sig(id: i64, reason: &str) -> SignatureEntry {
        SignatureEntry {
            sig: ElectronicSignature {
                id,
                user_id: "alice".into(),
                action: "sign".into(),
                resource_type: "batch".into(),
                resource_id: "B-114".into(),
                reason: reason.into(),
                meaning: "Approved".into(),
                signature_hash: "ab".repeat(32),
                created_at: 1_794_800_000,
            },
            full_name: Some("Alice Kusuma".into()),
            name_is_current: false,
            hash_valid: id != 2,
        }
    }

    fn text(pdf: &[u8]) -> String {
        String::from_utf8_lossy(pdf).into_owned()
    }

    fn pages(pdf: &[u8]) -> usize {
        text(pdf).matches("/Type /Page ").count()
    }

    #[test]
    fn times_carry_their_zone() {
        assert_eq!(zone_label(420), "UTC+07:00");
        assert_eq!(zone_label(-210), "UTC-03:30");
        assert_eq!(zone_label(0), "UTC");
        assert_eq!(format_time(0, 0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_time(0, 420), "1970-01-01 07:00:00 UTC+07:00");
        assert_eq!(format_time(0, -60), "1969-12-31 23:00:00 UTC-01:00");
    }

    #[test]
    fn a_signature_shows_its_full_manifestation() {
        let pdf = signatures_pdf(&meta(), &[sig(1, "Batch reviewed"), sig(2, "Bad")]);
        let t = text(&pdf);
        assert!(pdf.starts_with(b"%PDF-1.4") && pdf.ends_with(b"%%EOF\n"));
        for want in [
            "Signature #1",
            "Alice Kusuma \\(alice\\)",
            "Approved",
            "Batch reviewed",
            "batch / B-114",
            "UTC+07:00",
            "verified",
            "DOES NOT VERIFY",
            "1 with a checksum that does not verify",
            "page 1 of 1",
            "/BaseFont /Courier",
        ] {
            assert!(t.contains(want), "missing {want}");
        }
        // The filter is in the header of the page, and the checksum in full.
        assert!(t.contains("Plant \\(A\\) - Electronic signature records - 2026-11-01"));
        assert!(t.contains(&"ab".repeat(32)));
    }

    #[test]
    fn many_records_break_into_numbered_pages_with_the_header_on_each() {
        let entries: Vec<_> = (1..=60)
            .map(|i| sig(i, &"a long reason ".repeat(20)))
            .collect();
        let pdf = signatures_pdf(&meta(), &entries);
        let n = pages(&pdf);
        assert!(n >= 5, "{n} pages");
        let t = text(&pdf);
        assert!(t.contains(&format!("page {n} of {n}")));
        assert_eq!(t.matches("- 2026-11-01 00:00 to").count(), n);
    }

    #[test]
    fn a_name_not_recorded_at_signing_is_marked() {
        let mut e = sig(1, "r");
        e.name_is_current = true;
        let t = text(&signatures_pdf(&meta(), &[e]));
        assert!(t.contains("not recorded at signing"));
        let mut e = sig(1, "r");
        e.full_name = None;
        let t = text(&signatures_pdf(&meta(), &[e]));
        assert!(t.contains("alice \\(no full name on file\\)"));
    }

    #[test]
    fn the_audit_trail_prints_entries_checksums_and_the_chain() {
        let entries = vec![
            AuditEntry {
                id: 7,
                created_at: 1_794_800_000,
                username: "alice".into(),
                action: "electronic_signature".into(),
                resource: "batch:B-114".into(),
                details: Some("meaning=Approved".into()),
                hash: Some("cd".repeat(32)),
            },
            AuditEntry {
                id: 8,
                created_at: 1_794_800_100,
                username: "bob".into(),
                action: "login".into(),
                resource: "auth".into(),
                details: None,
                hash: None,
            },
        ];
        let ok = ChainSummary {
            ok: true,
            checked: 2,
            unhashed_legacy: 1,
            first_bad_id: None,
        };
        let t = text(&audit_pdf(&meta(), &entries, &ok));
        for want in [
            "#7  ",
            "electronic_signature",
            "meaning=Approved",
            "(before the hash chain)",
            "verified: 2 entries intact, 1 older",
        ] {
            let want = want.replace('(', "\\(").replace(')', "\\)");
            assert!(t.contains(&want), "missing {want}");
        }
        assert!(t.contains(&"cd".repeat(32)));
        let broken = ChainSummary {
            ok: false,
            checked: 1,
            unhashed_legacy: 0,
            first_bad_id: Some(8),
        };
        assert!(text(&audit_pdf(&meta(), &entries, &broken)).contains("BROKEN at entry 8"));
        assert!(text(&audit_pdf(&meta(), &[], &ok)).contains("No audit entries"));
    }
}
