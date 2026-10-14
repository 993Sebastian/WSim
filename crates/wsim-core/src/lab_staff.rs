//! The staff of a laboratory by role (PD1, docs/FORMELN.md, PD1): the player (and the AI)
//! hires and dismisses people per role; the size of the laboratories limits the posts.
//! Until the development projects (PD2) the research effort follows the hired people.

use serde::{Deserialize, Serialize};

use crate::calendar::Date;
use crate::catalog::{Catalog, LabRole, SiteType};
use crate::command::CommandError;
use crate::state::{GameState, Site, SiteId};

/// People per role in a research center.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabStaff {
    pub counts: [u32; 4],
    /// People assigned to development projects (PD2); recomputed every day.
    #[serde(default)]
    pub assigned: [u32; 4],
}

impl LabStaff {
    pub fn is_empty(&self) -> bool {
        self.counts.iter().all(|&n| n == 0)
    }

    pub fn get(&self, role: LabRole) -> u32 {
        self.counts[role.index()]
    }

    /// People of a role not assigned to a project.
    pub fn free(&self, role: LabRole) -> u32 {
        self.counts[role.index()].saturating_sub(self.assigned[role.index()])
    }

    pub fn total(&self) -> u32 {
        self.counts.iter().sum()
    }

    /// The people weighted by their role in the research effort.
    pub fn effort(&self, catalog: &Catalog) -> f64 {
        let model = &catalog.research_model.lab_staff;
        LabRole::ALL
            .iter()
            .map(|&r| f64::from(self.free(r)) * model.weight(r).unwrap_or(0.0))
            .sum()
    }
}

/// The posts of a research center: the runs of its operating laboratories at full
/// utilization.
pub fn capacity(catalog: &Catalog, site: &Site, date: Date) -> f64 {
    if site.kind != SiteType::ResearchCenter {
        return 0.0;
    }
    site.slots
        .iter()
        .filter(|sl| sl.operating(date))
        .map(|sl| sl.full_runs(catalog))
        .sum()
}

/// Share of the researcher posts the hired people fill, weighted by role (1 without the
/// roles in the data: the posts fill themselves).
pub fn research_share(catalog: &Catalog, site: &Site, date: Date) -> f64 {
    if !catalog.research_model.lab_staff.enabled() {
        return 1.0;
    }
    let capacity = capacity(catalog, site, date);
    if capacity <= 0.0 {
        return 0.0;
    }
    (site.lab_staff.effort(catalog) / capacity).min(1.0)
}

/// People who cost wages but do not research, at the center (PD1): all the hired people
/// less those the research effort counts.
pub fn idle_people(catalog: &Catalog, site: &Site) -> f64 {
    if !catalog.research_model.lab_staff.enabled() {
        return 0.0;
    }
    (f64::from(site.lab_staff.total()) - site.lab_staff.effort(catalog)).max(0.0)
}

/// `HireLabStaff` and `DismissLabStaff`: change the number of people in a role.
pub(crate) fn change(
    state: &mut GameState,
    catalog: &Catalog,
    actor: crate::state::CompanyId,
    (site, role): (SiteId, LabRole),
    (count, hire): (u32, bool),
) -> Result<(), CommandError> {
    let date = state.date;
    let Some(s) = state.sites.get_mut(site.index()) else {
        return Err(CommandError::UnknownSite);
    };
    if !s.held_by(actor) {
        return Err(CommandError::NotOwner);
    }
    if s.kind != SiteType::ResearchCenter {
        return Err(CommandError::WrongSiteType {
            required: SiteType::ResearchCenter,
        });
    }
    if !catalog.research_model.lab_staff.enabled()
        || catalog.research_model.lab_staff.weight(role).is_none()
        || count == 0
    {
        return Err(CommandError::InvalidQuantity);
    }
    let have = s.lab_staff.get(role);
    if hire {
        let room = capacity(catalog, s, date).floor() as i64 - i64::from(s.lab_staff.total());
        if i64::from(count) > room.max(0) {
            return Err(CommandError::LabFull);
        }
        s.lab_staff.counts[role.index()] = have + count;
    } else {
        if count > have {
            return Err(CommandError::InvalidQuantity);
        }
        s.lab_staff.counts[role.index()] = have - count;
    }
    s.staffing_due = true;
    Ok(())
}

/// The people of a group who research at the center: the workforce less those hired
/// without a part in the research (PD1), who are counted in the first group of
/// researchers.
pub fn researchers(catalog: &Catalog, site: &Site, group: crate::ids::LaborGroupId) -> f64 {
    let workers = *site.workforce.get(group);
    let first = catalog
        .research_model
        .researchers
        .iter()
        .copied()
        .flatten()
        .next();
    if first == Some(group) {
        (workers - idle_people(catalog, site)).max(0.0)
    } else {
        workers
    }
}
