//! A small PDF writer for the scheduled reports: A4 pages, the standard
//! Helvetica fonts (no font files embedded), text, rules and trend lines.
//! Text outside Latin-1 prints as `?`.

use super::{ReportData, capitalize_first, format_timestamp};

const PAGE_W: f64 = 595.0;
const PAGE_H: f64 = 842.0;
const MARGIN: f64 = 48.0;
const BOTTOM: f64 = 64.0;

struct Writer {
    pages: Vec<String>,
    page: String,
    y: f64,
}

/// Escape for a PDF literal string, mapping to WinAnsi (Latin-1) bytes.
fn pdf_text(s: &str, max_chars: usize) -> Vec<u8> {
    let mut chars: Vec<char> = s.chars().filter(|c| !c.is_control()).collect();
    if chars.len() > max_chars {
        chars.truncate(max_chars.saturating_sub(3));
        chars.extend(['.', '.', '.']);
    }
    let mut out = Vec::with_capacity(chars.len());
    for c in chars {
        let b = if (c as u32) < 0x100 {
            c as u32 as u8
        } else {
            b'?'
        };
        if matches!(b, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(b);
    }
    out
}

impl Writer {
    fn new() -> Self {
        Self {
            pages: Vec::new(),
            page: String::new(),
            y: PAGE_H - MARGIN,
        }
    }

    fn new_page(&mut self) {
        let page = std::mem::take(&mut self.page);
        self.pages.push(page);
        self.y = PAGE_H - MARGIN;
    }

    /// Start a new page unless `height` more points fit on this one.
    fn need(&mut self, height: f64) {
        if self.y - height < BOTTOM {
            self.new_page();
        }
    }

    fn text(&mut self, x: f64, y: f64, size: f64, bold: bool, s: &str, max_chars: usize) {
        let font = if bold { "F2" } else { "F1" };
        // The text bytes go in as Latin-1; the stream is assembled as bytes later.
        let escaped = pdf_text(s, max_chars);
        self.page
            .push_str(&format!("BT /{font} {size} Tf {x:.1} {y:.1} Td ("));
        // Keep Latin-1 bytes intact by storing them as chars U+0000..U+00FF.
        self.page.extend(escaped.iter().map(|&b| b as char));
        self.page.push_str(") Tj ET\n");
    }

    fn rule(&mut self, y: f64, gray: f64) {
        self.page.push_str(&format!(
            "{gray:.2} G 0.5 w {MARGIN:.1} {y:.1} m {:.1} {y:.1} l S 0 G\n",
            PAGE_W - MARGIN
        ));
    }

    /// A trend line of `values` in the box (x, y) to (x + w, y + h).
    fn sparkline(&mut self, x: f64, y: f64, w: f64, h: f64, values: &[f64]) {
        if values.len() < 2 {
            return;
        }
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = if (max - min).abs() < f64::EPSILON {
            1.0
        } else {
            max - min
        };
        let step = w / (values.len() - 1) as f64;
        self.page.push_str("0.23 0.51 0.96 RG 0.8 w ");
        for (i, v) in values.iter().enumerate() {
            let px = x + step * i as f64;
            let py = y + (v - min) / range * h;
            let op = if i == 0 { "m" } else { "l" };
            self.page.push_str(&format!("{px:.1} {py:.1} {op} "));
        }
        self.page.push_str("S 0 G\n");
    }

    fn heading(&mut self, s: &str) {
        self.need(40.0);
        self.y -= 26.0;
        self.text(MARGIN, self.y, 13.0, true, s, 80);
        self.y -= 6.0;
        self.rule(self.y, 0.6);
        self.y -= 14.0;
    }

    fn finish(mut self, footer: &str) -> Vec<u8> {
        if !self.page.is_empty() || self.pages.is_empty() {
            self.new_page();
        }
        let n = self.pages.len();
        // Objects: 1 catalog, 2 pages, 3 Helvetica, 4 Helvetica-Bold, then per page
        // a page object and its content stream.
        let mut objects: Vec<Vec<u8>> = vec![
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            format!(
                "<< /Type /Pages /Kids [{}] /Count {n} >>",
                (0..n)
                    .map(|i| format!("{} 0 R", 5 + i * 2))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .into_bytes(),
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
                .to_vec(),
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
                .to_vec(),
        ];
        for (i, body) in self.pages.iter().enumerate() {
            let mut content: Vec<u8> = body.chars().map(|c| c as u32 as u8).collect();
            let foot = format!("{footer} - page {} of {n}", i + 1);
            content.extend_from_slice(b"0.4 g ");
            content.extend_from_slice(format!("BT /F1 8 Tf {MARGIN:.1} 36 Td (").as_bytes());
            content.extend(pdf_text(&foot, 120));
            content.extend_from_slice(b") Tj ET 0 g\n");
            objects.push(
                format!(
                    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                    6 + i * 2
                )
                .into_bytes(),
            );
            let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
            stream.extend(content);
            stream.extend_from_slice(b"\nendstream");
            objects.push(stream);
        }

        let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
        let mut offsets = Vec::with_capacity(objects.len());
        for (i, obj) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(obj);
            out.extend_from_slice(b"\nendobj\n");
        }
        let xref = out.len();
        out.extend_from_slice(
            format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
        );
        for off in offsets {
            out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        out
    }
}

/// The report as a PDF document.
pub fn render_pdf(data: &ReportData) -> Vec<u8> {
    let mut w = Writer::new();
    let title_y = w.y - 10.0;
    w.text(MARGIN, title_y, 20.0, true, &data.name, 48);
    w.y = title_y - 20.0;
    let period = format!(
        "{} report, {} to {} (UTC)",
        capitalize_first(&data.schedule),
        format_timestamp(data.from),
        format_timestamp(data.to)
    );
    w.text(MARGIN, w.y, 10.0, false, &period, 100);
    w.y -= 16.0;
    let summary = format!(
        "Devices: {}    Metrics: {}    Alarms in period: {}",
        data.device_count,
        data.metrics.len(),
        data.alarms.len()
    );
    w.text(MARGIN, w.y, 10.0, false, &summary, 100);

    w.heading("Device metrics");
    if data.metrics.is_empty() {
        w.text(
            MARGIN,
            w.y,
            10.0,
            false,
            "No telemetry data for the reporting period.",
            100,
        );
        w.y -= 14.0;
    } else {
        let cols = [MARGIN, 160.0, 290.0, 345.0, 400.0, 455.0];
        let header = |w: &mut Writer| {
            for (x, h) in cols
                .iter()
                .zip(["Device", "Metric", "Min", "Max", "Avg", "Trend"])
            {
                w.text(*x, w.y, 8.0, true, h, 20);
            }
            w.y -= 12.0;
        };
        header(&mut w);
        for m in &data.metrics {
            if w.y - 14.0 < BOTTOM {
                w.new_page();
                header(&mut w);
            }
            w.text(cols[0], w.y, 9.0, false, &m.device_id, 20);
            w.text(cols[1], w.y, 9.0, false, &m.metric, 23);
            w.text(cols[2], w.y, 9.0, false, &format!("{:.2}", m.min), 10);
            w.text(cols[3], w.y, 9.0, false, &format!("{:.2}", m.max), 10);
            w.text(cols[4], w.y, 9.0, false, &format!("{:.2}", m.avg), 10);
            w.sparkline(cols[5], w.y - 1.0, PAGE_W - MARGIN - cols[5], 9.0, &m.trend);
            w.y -= 14.0;
        }
    }

    w.heading("Alarm log");
    if data.alarms.is_empty() {
        w.text(
            MARGIN,
            w.y,
            10.0,
            false,
            "No alarms during the reporting period.",
            100,
        );
        w.y -= 14.0;
    } else {
        let cols = [MARGIN, 160.0, 215.0, 330.0, 500.0];
        let header = |w: &mut Writer| {
            for (x, h) in cols
                .iter()
                .zip(["Time (UTC)", "Severity", "Source", "Message", "Value"])
            {
                w.text(*x, w.y, 8.0, true, h, 20);
            }
            w.y -= 12.0;
        };
        header(&mut w);
        for a in &data.alarms {
            if w.y - 14.0 < BOTTOM {
                w.new_page();
                header(&mut w);
            }
            w.text(
                cols[0],
                w.y,
                8.0,
                false,
                &format_timestamp(a.triggered_at),
                25,
            );
            w.text(cols[1], w.y, 8.0, false, &a.severity.to_uppercase(), 10);
            w.text(cols[2], w.y, 8.0, false, &a.source_name, 22);
            w.text(cols[3], w.y, 8.0, false, &a.message, 34);
            w.text(cols[4], w.y, 8.0, false, &format!("{:.2}", a.value), 12);
            w.y -= 13.0;
        }
    }

    w.finish(&format!(
        "Voltrus SCADA - generated {}",
        format_timestamp(data.to)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reports::render::{AlarmRow, MetricRow};

    fn data(metrics: usize, alarms: usize) -> ReportData {
        ReportData {
            name: "Daily (summary) \\ Plant A".into(),
            schedule: "daily".into(),
            from: 1_700_000_000,
            to: 1_700_086_400,
            device_count: 1,
            metrics: (0..metrics)
                .map(|i| MetricRow {
                    device_id: "plc1".into(),
                    metric: format!("temp_{i}"),
                    min: 1.0,
                    max: 9.0,
                    avg: 5.0,
                    samples: 3,
                    trend: vec![1.0, 9.0, 5.0],
                    ascii_trend: String::new(),
                })
                .collect(),
            alarms: (0..alarms)
                .map(|i| AlarmRow {
                    severity: "critical".into(),
                    source_name: "Tank 1 level".into(),
                    message: format!("High level {i} (Ünïcode ✓)"),
                    value: 98.5,
                    triggered_at: 1_700_050_000,
                })
                .collect(),
        }
    }

    fn page_count(pdf: &[u8]) -> usize {
        let text = String::from_utf8_lossy(pdf);
        text.matches("/Type /Page ").count()
    }

    #[test]
    fn pdf_is_well_formed() {
        let pdf = render_pdf(&data(2, 2));
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let text = String::from_utf8_lossy(&pdf);
        // The xref offsets point at the objects.
        let xref_at: usize = text
            .rsplit("startxref\n")
            .next()
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(pdf[xref_at..].starts_with(b"xref"));
        // Offsets are bytes of the file; the xref section itself is ASCII.
        let tail = std::str::from_utf8(&pdf[xref_at..]).unwrap();
        let entries: Vec<&str> = tail.lines().skip(3).take(4).collect();
        for (i, e) in entries.iter().enumerate() {
            let off: usize = e[..10].parse().unwrap();
            assert!(pdf[off..].starts_with(format!("{} 0 obj", i + 1).as_bytes()));
        }
        // Parentheses and backslashes in text are escaped.
        assert!(text.contains("Daily \\(summary\\) \\\\ Plant A"));
        assert!(text.contains("Device metrics"));
        assert!(text.contains("Alarm log"));
        // Latin-1 survives as one byte, other characters become '?'.
        assert!(pdf.windows(2).any(|w| w == [b'\xDC', b'n']));
        assert!(text.contains("code ?\\)"));
        assert_eq!(page_count(&pdf), 1);
    }

    #[test]
    fn long_reports_break_into_pages() {
        let pdf = render_pdf(&data(80, 120));
        assert!(page_count(&pdf) >= 4);
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("page 1 of"));
    }

    #[test]
    fn empty_report_still_renders() {
        let pdf = render_pdf(&data(0, 0));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("No telemetry data"));
        assert!(text.contains("No alarms during"));
        assert_eq!(page_count(&pdf), 1);
    }
}
