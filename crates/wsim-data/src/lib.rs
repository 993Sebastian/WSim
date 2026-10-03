//! Loading and validation of the game data in `data/`.
//!
//! Reads the YAML files, checks them and compiles them into the catalog used by
//! `wsim-core`. Problems are collected in a [`Report`] with file, line and a German
//! message, so that data can be edited without programming knowledge (Lastenheft §16.2).

mod build;
mod de;
mod messages;
mod raw;
mod read;
mod report;
mod suggest;
mod texts;
mod yaml;

use std::fs;
use std::path::Path as FsPath;

use wsim_core::catalog::Catalog;

pub use report::{Finding, Path, Report, Segment, Severity};
pub use texts::Texts;
pub use yaml::Position;

use read::{Ctx, RawData, SourceFile};
use texts::TextIndex;

/// Language of the display texts (`data/texte/<LANGUAGE>/`).
pub const LANGUAGE: &str = "de";

/// Checked game content.
#[derive(Debug, Clone)]
pub struct GameData {
    pub catalog: Catalog,
    pub texts: Texts,
}

/// Result of loading: the content if there were no errors, and all findings.
#[derive(Debug)]
pub struct LoadOutcome {
    pub data: Option<GameData>,
    pub report: Report,
}

/// A data file given in memory, with its path relative to the data directory.
#[derive(Debug, Clone)]
pub struct Source {
    pub path: String,
    pub text: String,
}

impl Source {
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}

/// Loads all data below `root` (usually the `data/` directory).
pub fn load_dir(root: &FsPath) -> LoadOutcome {
    let prefix = root
        .to_string_lossy()
        .trim_end_matches(['/', '\\'])
        .replace('\\', "/");
    let mut report = Report::default();
    let mut sources = Vec::new();
    if root.is_dir() {
        collect(root, "", &prefix, &mut sources, &mut report);
    } else {
        report.push(Finding {
            severity: Severity::Error,
            file: Some(prefix.clone()),
            position: None,
            path: Path::default(),
            message: messages::not_a_directory(),
        });
        return LoadOutcome { data: None, report };
    }
    let mut outcome = load_sources_with_prefix(sources, Some(&prefix));
    for finding in report.findings() {
        outcome.report.push(finding.clone());
    }
    outcome.report.sort();
    outcome
}

/// Loads data given in memory (used by tests and tools).
pub fn load_sources(sources: Vec<Source>) -> LoadOutcome {
    load_sources_with_prefix(sources, None)
}

fn load_sources_with_prefix(mut sources: Vec<Source>, prefix: Option<&str>) -> LoadOutcome {
    // Sorted paths give stable IDs and messages on every system.
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    let text_dir = format!("texte/{LANGUAGE}/");

    let mut files = Vec::new();
    let mut syntax_errors = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        let display = match prefix {
            Some(prefix) => format!("{prefix}/{}", source.path),
            None => source.path.clone(),
        };
        let root = match yaml::parse(&source.text) {
            Ok(root) => Some(root),
            Err(e) => {
                syntax_errors.push((index, e));
                None
            }
        };
        files.push(SourceFile { display, root });
    }

    let mut ctx = Ctx {
        files: &files,
        report: Report::default(),
    };
    // Unreadable files hide entries, so texts for them would look unused.
    let all_files_read = syntax_errors.is_empty();
    for (index, error) in syntax_errors {
        ctx.file_error(index, Some(error.position), error.message);
    }

    let mut raw = RawData::default();
    let mut texts = TextIndex::default();
    let mut has_text_dir = false;
    for (index, source) in sources.iter().enumerate() {
        if source.path.starts_with(&text_dir) {
            has_text_dir = true;
            texts::read_text_file(&mut ctx, index, &mut texts);
        } else if !source.path.starts_with("texte/") {
            read::read_content_file(&mut ctx, index, &mut raw);
        }
    }
    if !has_text_dir {
        ctx.general_error(messages::texts_missing(LANGUAGE));
    }

    let catalog = build::build(&mut ctx, &raw, &texts, all_files_read);
    let mut report = ctx.report;
    report.sort();
    let data = catalog
        .filter(|_| !report.has_errors())
        .map(|catalog| GameData {
            catalog,
            texts: texts.texts,
        });
    LoadOutcome { data, report }
}

/// Collects `*.yaml` files below `dir` with paths relative to the data root.
fn collect(dir: &FsPath, relative: &str, prefix: &str, out: &mut Vec<Source>, report: &mut Report) {
    let mut entries = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(e) => {
            report.push(file_finding(
                format!("{prefix}/{relative}"),
                messages::read_error(e),
            ));
            return;
        }
    };
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if relative.is_empty() {
            name.clone()
        } else {
            format!("{relative}/{name}")
        };
        let file_type = entry.file_type();
        if file_type.as_ref().is_ok_and(fs::FileType::is_dir) {
            collect(&entry.path(), &path, prefix, out, report);
        } else if name.ends_with(".yaml") {
            match fs::read_to_string(entry.path()) {
                Ok(text) => out.push(Source { path, text }),
                Err(e) => report.push(file_finding(
                    format!("{prefix}/{path}"),
                    messages::read_error(e),
                )),
            }
        } else if name.ends_with(".yml") {
            let mut finding = file_finding(format!("{prefix}/{path}"), messages::yml_extension());
            finding.severity = Severity::Warning;
            report.push(finding);
        }
    }
}

fn file_finding(file: String, message: String) -> Finding {
    Finding {
        severity: Severity::Error,
        file: Some(file),
        position: None,
        path: Path::default(),
        message,
    }
}
