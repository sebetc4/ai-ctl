//! Diagnostics: the checks `doctor` reports, grouped in sections.

/// Outcome of one check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The requirement is met.
    Ok,
    /// Something works but deserves attention; it does not fail the run.
    Warn,
    /// A requirement is not met; the run fails.
    Fail,
    /// The check could not apply, because something it depends on is absent.
    Skip,
}

impl Status {
    /// Lower-case name, as printed and serialized.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Fail => "fail",
            Status::Skip => "skip",
        }
    }
}

/// One diagnosed item: what was checked, what was found, and how to fix it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: String,
    pub status: Status,
    pub detail: String,
    /// Commands or instructions that fix the problem; one step per line.
    pub remedy: Option<String>,
}

impl Check {
    pub fn ok(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(name, Status::Ok, detail, None)
    }

    pub fn skip(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(name, Status::Skip, detail, None)
    }

    pub fn warn(
        name: impl Into<String>,
        detail: impl Into<String>,
        remedy: impl Into<Option<String>>,
    ) -> Self {
        Self::new(name, Status::Warn, detail, remedy.into())
    }

    pub fn fail(
        name: impl Into<String>,
        detail: impl Into<String>,
        remedy: impl Into<Option<String>>,
    ) -> Self {
        Self::new(name, Status::Fail, detail, remedy.into())
    }

    fn new(
        name: impl Into<String>,
        status: Status,
        detail: impl Into<String>,
        remedy: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            status,
            detail: detail.into(),
            remedy,
        }
    }
}

/// A named group of checks, such as "Toolchain".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub checks: Vec<Check>,
}

impl Section {
    pub fn new(name: impl Into<String>, checks: Vec<Check>) -> Self {
        Self {
            name: name.into(),
            checks,
        }
    }
}

/// Number of checks per status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Summary {
    pub ok: usize,
    pub warn: usize,
    pub fail: usize,
    pub skip: usize,
}

/// Every section of a diagnosis.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub sections: Vec<Section>,
}

impl Report {
    pub fn checks(&self) -> impl Iterator<Item = &Check> {
        self.sections.iter().flat_map(|section| &section.checks)
    }

    pub fn summary(&self) -> Summary {
        let mut summary = Summary::default();
        for check in self.checks() {
            match check.status {
                Status::Ok => summary.ok += 1,
                Status::Warn => summary.warn += 1,
                Status::Fail => summary.fail += 1,
                Status::Skip => summary.skip += 1,
            }
        }
        summary
    }

    /// True when at least one check failed: the run exits with an error.
    pub fn has_failures(&self) -> bool {
        self.checks().any(|check| check.status == Status::Fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(statuses: &[&[Status]]) -> Report {
        Report {
            sections: statuses
                .iter()
                .enumerate()
                .map(|(i, section)| {
                    let checks = section
                        .iter()
                        .map(|&status| Check::new("check", status, "detail", None))
                        .collect();
                    Section::new(format!("section {i}"), checks)
                })
                .collect(),
        }
    }

    #[test]
    fn summary_counts_every_status_across_sections() {
        let report = report(&[
            &[Status::Ok, Status::Ok, Status::Warn],
            &[Status::Fail, Status::Skip, Status::Ok],
        ]);
        assert_eq!(
            report.summary(),
            Summary {
                ok: 3,
                warn: 1,
                fail: 1,
                skip: 1
            }
        );
    }

    #[test]
    fn an_empty_report_has_no_failure() {
        let report = Report::default();
        assert_eq!(report.summary(), Summary::default());
        assert!(!report.has_failures());
    }

    #[test]
    fn warnings_and_skips_do_not_fail_the_run() {
        assert!(!report(&[&[Status::Ok, Status::Warn, Status::Skip]]).has_failures());
    }

    #[test]
    fn one_failure_fails_the_run() {
        assert!(report(&[&[Status::Ok], &[Status::Warn, Status::Fail]]).has_failures());
    }

    #[test]
    fn constructors_set_status_and_remedy() {
        let check = Check::fail(
            "CMake",
            "not found",
            Some("sudo dnf install cmake".to_owned()),
        );
        assert_eq!(check.status, Status::Fail);
        assert_eq!(check.remedy.as_deref(), Some("sudo dnf install cmake"));
        assert_eq!(Check::ok("Git", "2.56.0").remedy, None);
        assert_eq!(Check::skip("CDI", "no nvidia-ctk").status, Status::Skip);
        assert_eq!(Check::warn("Podman", "missing", None).status, Status::Warn);
    }

    #[test]
    fn status_names_are_lower_case() {
        let names: Vec<_> = [Status::Ok, Status::Warn, Status::Fail, Status::Skip]
            .into_iter()
            .map(Status::as_str)
            .collect();
        assert_eq!(names, ["ok", "warn", "fail", "skip"]);
    }
}
