//! Commands: the only way to change a company (Lastenheft §10).
//!
//! The player, AI companies and later managers all act through the same commands and
//! the same checks.

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::message::{Message, Param, keys};
use crate::state::{CompanyId, GameState};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// Renames the acting company.
    RenameCompany { name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NameError {
    Empty,
    TooLong { max: usize },
    Taken { name: String },
}

/// Longest allowed company name in characters.
pub const MAX_NAME_LENGTH: usize = 60;

impl NameError {
    pub fn message(&self) -> Message {
        match self {
            NameError::Empty => Message::error(keys::NAME_EMPTY),
            NameError::TooLong { max } => Message::error(keys::NAME_TOO_LONG).with(
                "max",
                Param::Integer(i64::try_from(*max).unwrap_or(i64::MAX)),
            ),
            NameError::Taken { name } => {
                Message::error(keys::NAME_TAKEN).with("name", Param::Text(name.clone()))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandError {
    GameOver,
    UnknownCompany(CompanyId),
    Name(NameError),
}

impl CommandError {
    pub fn message(&self) -> Message {
        match self {
            CommandError::GameOver => Message::error(keys::COMMAND_GAME_OVER),
            CommandError::UnknownCompany(id) => Message::error(keys::COMMAND_UNKNOWN_COMPANY)
                .with("firma", Param::Integer(i64::from(id.0))),
            CommandError::Name(e) => e.message(),
        }
    }
}

/// Checks a company name and returns it trimmed. `own` is excluded from the
/// uniqueness check (renaming to the same name is fine).
pub fn check_company_name(
    state: Option<&GameState>,
    name: &str,
    own: Option<CompanyId>,
) -> Result<String, NameError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().count() > MAX_NAME_LENGTH {
        return Err(NameError::TooLong {
            max: MAX_NAME_LENGTH,
        });
    }
    if let Some(state) = state {
        let taken = state.companies.iter().enumerate().any(|(i, c)| {
            Some(CompanyId(u32::try_from(i).unwrap_or(u32::MAX))) != own
                && c.name.eq_ignore_ascii_case(name)
        });
        if taken {
            return Err(NameError::Taken {
                name: name.to_owned(),
            });
        }
    }
    Ok(name.to_owned())
}

pub(crate) fn execute(
    state: &mut GameState,
    _catalog: &Catalog,
    actor: CompanyId,
    command: &Command,
) -> Result<(), CommandError> {
    if state.game_over {
        return Err(CommandError::GameOver);
    }
    if state.company(actor).is_none() {
        return Err(CommandError::UnknownCompany(actor));
    }
    match command {
        Command::RenameCompany { name } => {
            let name =
                check_company_name(Some(state), name, Some(actor)).map_err(CommandError::Name)?;
            state.company_mut(actor).expect("checked above").name = name;
        }
    }
    Ok(())
}
