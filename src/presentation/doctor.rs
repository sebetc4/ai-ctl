//! `ai-ctl doctor`: renders the diagnosis as text or JSON.

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::domain::diagnostics::{Report, Status};

const INDENT: &str = "  ";
const STATUS_WIDTH: usize = 4;
const REMEDY_LABEL: &str = "fix: ";

/// Renders `report` as one line per check under each section, remedies
/// below the check they fix, and the summary counts last. `color` adds ANSI
/// colors to the statuses.
pub fn render_text(report: &Report, color: bool) -> String {
    let name_width = report
        .checks()
        .map(|check| check.name.chars().count())
        .max()
        .unwrap_or(0);
    // Remedy lines start under the detail column.
    let remedy_indent = " ".repeat(INDENT.len() + STATUS_WIDTH + 2 + name_width + 2);

    let mut out = String::new();
    for (i, section) in report.sections.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let _ = writeln!(out, "{}", paint(&section.name, BOLD, color));
        for check in &section.checks {
            let status = format!("{:<STATUS_WIDTH$}", check.status.as_str());
            let _ = writeln!(
                out,
                "{INDENT}{}  {:<name_width$}  {}",
                paint(&status, status_style(check.status), color),
                check.name,
                check.detail
            );
            if let Some(remedy) = &check.remedy {
                for (line_no, line) in remedy.lines().enumerate() {
                    let label = if line_no == 0 { REMEDY_LABEL } else { "     " };
                    let _ = writeln!(out, "{remedy_indent}{label}{line}");
                }
            }
        }
    }

    let summary = report.summary();
    let _ = writeln!(
        out,
        "\n{} ok, {} warn, {} fail, {} skip",
        summary.ok, summary.warn, summary.fail, summary.skip
    );
    out
}

/// Renders `report` as a JSON document holding every check of the text
/// output, with the summary counts and an overall verdict.
pub fn render_json(report: &Report) -> Value {
    let summary = report.summary();
    let sections: Vec<Value> = report
        .sections
        .iter()
        .map(|section| {
            let checks: Vec<Value> = section
                .checks
                .iter()
                .map(|check| {
                    json!({
                        "name": check.name,
                        "status": check.status.as_str(),
                        "detail": check.detail,
                        "remedy": check.remedy,
                    })
                })
                .collect();
            json!({ "name": section.name, "checks": checks })
        })
        .collect();
    json!({
        "healthy": !report.has_failures(),
        "summary": {
            "ok": summary.ok,
            "warn": summary.warn,
            "fail": summary.fail,
            "skip": summary.skip,
        },
        "sections": sections,
    })
}

const BOLD: &str = "1";

fn status_style(status: Status) -> &'static str {
    match status {
        Status::Ok => "32",
        Status::Warn => "33",
        Status::Fail => "1;31",
        Status::Skip => "2",
    }
}

fn paint(text: &str, style: &str, color: bool) -> String {
    if color {
        format!("\x1b[{style}m{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::diagnostics::{Check, Section};

    fn report() -> Report {
        Report {
            sections: vec![
                Section::new(
                    "Toolchain",
                    vec![
                        Check::fail(
                            "CUDA toolkit",
                            "13.2.86: reported to produce wrong output",
                            Some(
                                "sudo dnf config-manager addrepo --from-repofile=x.repo\n\
                                 sudo dnf install cuda-toolkit-13-4"
                                    .to_owned(),
                            ),
                        ),
                        Check::ok("CMake", "4.3.0"),
                    ],
                ),
                Section::new(
                    "Power",
                    vec![
                        Check::warn("AC power", "on battery", Some("plug in".to_owned())),
                        Check::skip("Power profile", "no service"),
                    ],
                ),
            ],
        }
    }

    #[test]
    fn text_has_one_aligned_line_per_check_and_remedies_below() {
        let text = render_text(&report(), false);
        let expected = "\
Toolchain
  fail  CUDA toolkit   13.2.86: reported to produce wrong output
                       fix: sudo dnf config-manager addrepo --from-repofile=x.repo
                            sudo dnf install cuda-toolkit-13-4
  ok    CMake          4.3.0

Power
  warn  AC power       on battery
                       fix: plug in
  skip  Power profile  no service

1 ok, 1 warn, 1 fail, 1 skip
";
        assert_eq!(text, expected);
    }

    #[test]
    fn color_wraps_statuses_only_when_asked() {
        assert!(!render_text(&report(), false).contains('\x1b'));
        let colored = render_text(&report(), true);
        assert!(colored.contains("\x1b[1;31mfail\x1b[0m"), "{colored}");
        assert!(colored.contains("\x1b[32mok  \x1b[0m"), "{colored}");
    }

    #[test]
    fn json_holds_the_same_checks_as_the_text() {
        let report = report();
        let json = render_json(&report);
        let text = render_text(&report, false);

        let checks: Vec<(&str, &str, &str, Option<&str>)> = json["sections"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|section| section["checks"].as_array().unwrap())
            .map(|check| {
                (
                    check["name"].as_str().unwrap(),
                    check["status"].as_str().unwrap(),
                    check["detail"].as_str().unwrap(),
                    check["remedy"].as_str(),
                )
            })
            .collect();
        let expected: Vec<_> = report
            .checks()
            .map(|check| {
                (
                    check.name.as_str(),
                    check.status.as_str(),
                    check.detail.as_str(),
                    check.remedy.as_deref(),
                )
            })
            .collect();
        assert_eq!(checks, expected);
        for (name, status, detail, _) in &checks {
            let line = text
                .lines()
                .find(|line| line.contains(detail))
                .unwrap_or_else(|| panic!("no text line for {name}"));
            assert!(line.contains(name) && line.contains(status), "{line}");
        }
    }

    #[test]
    fn json_carries_the_verdict_and_the_counts() {
        let json = render_json(&report());
        assert_eq!(json["healthy"], false);
        assert_eq!(
            json["summary"],
            json!({ "ok": 1, "warn": 1, "fail": 1, "skip": 1 })
        );
        assert_eq!(json["sections"][1]["checks"][1]["remedy"], Value::Null);
    }

    #[test]
    fn an_empty_report_renders() {
        let report = Report::default();
        assert_eq!(
            render_text(&report, false),
            "\n0 ok, 0 warn, 0 fail, 0 skip\n"
        );
        assert_eq!(render_json(&report)["healthy"], true);
    }
}
