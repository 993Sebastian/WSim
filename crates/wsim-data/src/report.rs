//! Findings of a data check and how they are presented.

use std::fmt;

use crate::yaml::Position;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

/// Location inside a data file, e.g. `rezepte[3].eingang.roheisn`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(Vec<Segment>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Segment {
    /// Value stored under a map key.
    Field(String),
    /// Element of a list.
    Index(usize),
    /// The map key itself (used when the key is a reference, e.g. recipe inputs).
    Key(String),
}

impl Path {
    pub fn segments(&self) -> &[Segment] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn field(&self, name: &str) -> Path {
        self.with(Segment::Field(name.to_owned()))
    }

    #[must_use]
    pub fn index(&self, index: usize) -> Path {
        self.with(Segment::Index(index))
    }

    #[must_use]
    pub fn key(&self, key: &str) -> Path {
        self.with(Segment::Key(key.to_owned()))
    }

    fn with(&self, segment: Segment) -> Path {
        let mut segments = self.0.clone();
        segments.push(segment);
        Path(segments)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            match segment {
                Segment::Index(index) => write!(f, "[{index}]")?,
                Segment::Field(name) | Segment::Key(name) => {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    f.write_str(name)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    /// File as shown to the user, e.g. `data/ketten/01_eisen_stahl.yaml`.
    pub file: Option<String>,
    pub position: Option<Position>,
    pub path: Path,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.severity {
            Severity::Error => "Fehler",
            Severity::Warning => "Warnung",
        };
        f.write_str(label)?;
        if let Some(file) = &self.file {
            write!(f, " in {file}")?;
            if let Some(position) = self.position {
                write!(f, ", Zeile {}", position.line)?;
            }
        }
        if !self.path.is_empty() {
            write!(f, " ({})", self.path)?;
        }
        write!(f, ":\n  {}", self.message)
    }
}

/// All findings of one check, sorted by file and line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    findings: Vec<Finding>,
}

impl Report {
    pub fn push(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    pub fn errors(&self) -> impl Iterator<Item = &Finding> {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &Finding> {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
    }

    pub fn has_errors(&self) -> bool {
        self.errors().next().is_some()
    }

    pub(crate) fn sort(&mut self) {
        self.findings.sort_by(|a, b| {
            (&a.file, a.position, a.severity, &a.message)
                .cmp(&(&b.file, b.position, b.severity, &b.message))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_paths() {
        let path = Path::default()
            .field("rezepte")
            .index(3)
            .field("eingang")
            .key("roheisn");
        assert_eq!(path.to_string(), "rezepte[3].eingang.roheisn");
    }

    #[test]
    fn formats_findings() {
        let finding = Finding {
            severity: Severity::Error,
            file: Some("data/ketten/a.yaml".into()),
            position: Some(Position {
                line: 12,
                column: 3,
            }),
            path: Path::default().field("produkte").index(0),
            message: "Etwas stimmt nicht.".into(),
        };
        assert_eq!(
            finding.to_string(),
            "Fehler in data/ketten/a.yaml, Zeile 12 (produkte[0]):\n  Etwas stimmt nicht."
        );
    }
}
