//! Reading parsed files into raw entries, and the shared context for findings.

use std::collections::{BTreeMap, BTreeSet};

use serde::de::DeserializeOwned;

use crate::de::{self, DeError};
use crate::messages;
use crate::raw::{
    RawAiModel, RawCountry, RawCountryModel, RawDeposit, RawEvent, RawFacility, RawFinanceModel,
    RawMarketModel, RawMeta, RawNameGroup, RawProduct, RawProductionModel, RawQualification,
    RawRealCompany, RawRecipe, RawResearchModel, RawSimple, RawTechnology, RawTransportClass,
    RawTransportModel, RawUnit, RawVehicle,
};
use crate::report::{Finding, Path, Report, Segment, Severity};
use crate::suggest;
use crate::yaml::{Kind, Node, Position};

pub(crate) struct SourceFile {
    pub display: String,
    /// `None` if the file could not be parsed.
    pub root: Option<Node>,
}

/// A location in one of the source files.
#[derive(Clone, Debug)]
pub(crate) struct Loc {
    pub file: usize,
    pub path: Path,
}

impl Loc {
    pub fn field(&self, name: &str) -> Loc {
        Loc {
            file: self.file,
            path: self.path.field(name),
        }
    }

    pub fn key(&self, name: &str) -> Loc {
        Loc {
            file: self.file,
            path: self.path.key(name),
        }
    }

    pub fn index(&self, index: usize) -> Loc {
        Loc {
            file: self.file,
            path: self.path.index(index),
        }
    }
}

pub(crate) struct Ctx<'a> {
    pub files: &'a [SourceFile],
    pub report: Report,
}

impl Ctx<'_> {
    fn finding(&self, severity: Severity, loc: &Loc, message: String) -> Finding {
        let file = &self.files[loc.file];
        Finding {
            severity,
            file: Some(file.display.clone()),
            position: file.root.as_ref().map(|root| locate(root, &loc.path)),
            path: loc.path.clone(),
            message,
        }
    }

    pub fn error(&mut self, loc: &Loc, message: String) {
        let finding = self.finding(Severity::Error, loc, message);
        self.report.push(finding);
    }

    pub fn warning(&mut self, loc: &Loc, message: String) {
        let finding = self.finding(Severity::Warning, loc, message);
        self.report.push(finding);
    }

    /// Error without a file, e.g. a missing directory.
    pub fn general_error(&mut self, message: String) {
        self.report.push(Finding {
            severity: Severity::Error,
            file: None,
            position: None,
            path: Path::default(),
            message,
        });
    }

    pub fn file_error(&mut self, file: usize, position: Option<Position>, message: String) {
        self.report.push(Finding {
            severity: Severity::Error,
            file: Some(self.files[file].display.clone()),
            position,
            path: Path::default(),
            message,
        });
    }

    /// "data/x.yaml, Zeile 5" for messages that refer to another place.
    pub fn describe(&self, loc: &Loc) -> String {
        let file = &self.files[loc.file];
        match file.root.as_ref().map(|root| locate(root, &loc.path)) {
            Some(position) => format!("{}, Zeile {}", file.display, position.line),
            None => file.display.clone(),
        }
    }

    fn de_error(&mut self, file: usize, error: DeError) {
        self.report.push(Finding {
            severity: Severity::Error,
            file: Some(self.files[file].display.clone()),
            position: error.position,
            path: error.path.unwrap_or_default(),
            message: error.message,
        });
    }
}

/// Position of the deepest node on `path` (falls back to the closest existing parent).
pub(crate) fn locate(root: &Node, path: &Path) -> Position {
    let mut node = root;
    for segment in path.segments() {
        let next = match (segment, &node.kind) {
            (Segment::Field(name), Kind::Map(_)) => node.get(name),
            (Segment::Key(name), Kind::Map(_)) => node.entry(name).map(|(key, _)| key),
            (Segment::Index(i), Kind::Seq(items)) => items.get(*i),
            _ => None,
        };
        match next {
            Some(next) => node = next,
            None => break,
        }
    }
    node.position
}

pub(crate) struct Entry<T> {
    pub loc: Loc,
    pub value: T,
}

#[derive(Default)]
pub(crate) struct RawData {
    /// IDs of entries that could not be read, by section. References to them are not
    /// reported again, the entry itself already has an error.
    pub broken: BTreeMap<String, BTreeSet<String>>,
    pub meta: Vec<Entry<RawMeta>>,
    pub country_model: Vec<Entry<RawCountryModel>>,
    pub production_model: Vec<Entry<RawProductionModel>>,
    pub finance_model: Vec<Entry<RawFinanceModel>>,
    pub market_model: Vec<Entry<RawMarketModel>>,
    pub transport_model: Vec<Entry<RawTransportModel>>,
    pub research_model: Vec<Entry<RawResearchModel>>,
    pub units: Vec<Entry<RawUnit>>,
    pub continents: Vec<Entry<RawSimple>>,
    pub branches: Vec<Entry<RawSimple>>,
    pub goods_groups: Vec<Entry<RawSimple>>,
    pub transport_classes: Vec<Entry<RawTransportClass>>,
    pub qualifications: Vec<Entry<RawQualification>>,
    pub specializations: Vec<Entry<RawSimple>>,
    pub countries: Vec<Entry<RawCountry>>,
    pub products: Vec<Entry<RawProduct>>,
    pub facilities: Vec<Entry<RawFacility>>,
    pub recipes: Vec<Entry<RawRecipe>>,
    pub technologies: Vec<Entry<RawTechnology>>,
    pub deposits: Vec<Entry<RawDeposit>>,
    pub vehicles: Vec<Entry<RawVehicle>>,
    pub ai_model: Vec<Entry<RawAiModel>>,
    pub name_groups: Vec<Entry<RawNameGroup>>,
    pub real_companies: Vec<Entry<RawRealCompany>>,
    pub events: Vec<Entry<RawEvent>>,
}

pub(crate) const SECTIONS: &[&str] = &[
    "meta",
    "laendermodell",
    "produktionsmodell",
    "finanzmodell",
    "marktmodell",
    "transportmodell",
    "forschungsmodell",
    "einheiten",
    "kontinente",
    "branchen",
    "warengruppen",
    "transportklassen",
    "qualifikationen",
    "fachrichtungen",
    "laender",
    "produkte",
    "anlagen",
    "rezepte",
    "technologien",
    "lagerstaetten",
    "verkehrsmittel",
    "kimodell",
    "namensgruppen",
    "reale_firmen",
    "ereignisse",
];

/// Reads all sections of one content file into `raw`.
pub(crate) fn read_content_file(ctx: &mut Ctx, file: usize, raw: &mut RawData) {
    let files = ctx.files;
    let Some(root) = &files[file].root else {
        return;
    };
    let file_loc = Loc {
        file,
        path: Path::default(),
    };
    let entries = match &root.kind {
        Kind::Null => return ctx.warning(&file_loc, messages::empty_file()),
        Kind::Map(entries) => entries,
        _ => return ctx.error(&file_loc, messages::file_not_a_map()),
    };
    for (key, value) in entries {
        let name = key.scalar_text().unwrap_or_default();
        let loc = file_loc.field(name);
        match name {
            "meta" => match de::from_node::<RawMeta>(value, &loc.path) {
                Ok(meta) => raw.meta.push(Entry { loc, value: meta }),
                Err(e) => ctx.de_error(file, e),
            },
            "laendermodell" => match de::from_node::<RawCountryModel>(value, &loc.path) {
                Ok(model) => raw.country_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "produktionsmodell" => match de::from_node::<RawProductionModel>(value, &loc.path) {
                Ok(model) => raw.production_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "finanzmodell" => match de::from_node::<RawFinanceModel>(value, &loc.path) {
                Ok(model) => raw.finance_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "marktmodell" => match de::from_node::<RawMarketModel>(value, &loc.path) {
                Ok(model) => raw.market_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "transportmodell" => match de::from_node::<RawTransportModel>(value, &loc.path) {
                Ok(model) => raw.transport_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "forschungsmodell" => match de::from_node::<RawResearchModel>(value, &loc.path) {
                Ok(model) => raw.research_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "kimodell" => match de::from_node::<RawAiModel>(value, &loc.path) {
                Ok(model) => raw.ai_model.push(Entry { loc, value: model }),
                Err(e) => ctx.de_error(file, e),
            },
            "namensgruppen" => read_list(ctx, &loc, value, &mut raw.name_groups, &mut raw.broken),
            "ereignisse" => read_list(ctx, &loc, value, &mut raw.events, &mut raw.broken),
            "reale_firmen" => read_list(ctx, &loc, value, &mut raw.real_companies, &mut raw.broken),
            "einheiten" => read_list(ctx, &loc, value, &mut raw.units, &mut raw.broken),
            "kontinente" => read_list(ctx, &loc, value, &mut raw.continents, &mut raw.broken),
            "branchen" => read_list(ctx, &loc, value, &mut raw.branches, &mut raw.broken),
            "warengruppen" => read_list(ctx, &loc, value, &mut raw.goods_groups, &mut raw.broken),
            "transportklassen" => {
                read_list(
                    ctx,
                    &loc,
                    value,
                    &mut raw.transport_classes,
                    &mut raw.broken,
                );
            }
            "qualifikationen" => {
                read_list(ctx, &loc, value, &mut raw.qualifications, &mut raw.broken)
            }
            "fachrichtungen" => {
                read_list(ctx, &loc, value, &mut raw.specializations, &mut raw.broken)
            }
            "laender" => read_list(ctx, &loc, value, &mut raw.countries, &mut raw.broken),
            "produkte" => read_list(ctx, &loc, value, &mut raw.products, &mut raw.broken),
            "anlagen" => read_list(ctx, &loc, value, &mut raw.facilities, &mut raw.broken),
            "rezepte" => read_list(ctx, &loc, value, &mut raw.recipes, &mut raw.broken),
            "technologien" => read_list(ctx, &loc, value, &mut raw.technologies, &mut raw.broken),
            "lagerstaetten" => read_list(ctx, &loc, value, &mut raw.deposits, &mut raw.broken),
            "verkehrsmittel" => read_list(ctx, &loc, value, &mut raw.vehicles, &mut raw.broken),
            other => ctx.error(
                &file_loc.key(other),
                messages::unknown_section(
                    other,
                    SECTIONS,
                    suggest::closest(other, SECTIONS.iter().copied()),
                ),
            ),
        }
    }
}

fn read_list<T: DeserializeOwned>(
    ctx: &mut Ctx,
    loc: &Loc,
    node: &Node,
    out: &mut Vec<Entry<T>>,
    broken: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let items = match &node.kind {
        Kind::Null => return,
        Kind::Seq(items) => items,
        _ => {
            let section = loc.path.to_string();
            return ctx.error(loc, messages::section_not_a_list(&section));
        }
    };
    for (index, item) in items.iter().enumerate() {
        let item_loc = loc.index(index);
        match de::from_node::<T>(item, &item_loc.path) {
            Ok(value) => out.push(Entry {
                loc: item_loc,
                value,
            }),
            Err(e) => {
                ctx.de_error(loc.file, e);
                if let Some(id) = item.get("id").and_then(Node::scalar_text) {
                    broken
                        .entry(loc.path.to_string())
                        .or_default()
                        .insert(id.to_owned());
                }
            }
        }
    }
}
