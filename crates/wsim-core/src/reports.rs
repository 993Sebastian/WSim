//! Financial reports from the ledger (Lastenheft §11.4): balance sheet, income
//! statement and cash flow.

use crate::ledger::{Account, CashFlow, CostType, Ledger, PeriodResult};
use crate::money::Money;

impl Account {
    pub fn text_key(self) -> &'static str {
        match self {
            Account::Cash => "konto.kasse",
            Account::Inventory => "konto.vorraete",
            Account::FixedAssets => "konto.sachanlagen",
            Account::AssetsUnderConstruction => "konto.anlagen_im_bau",
            Account::Loans => "konto.kredite",
            Account::Equity => "konto.eigenkapital",
            Account::RetainedEarnings => "konto.gewinnruecklagen",
            Account::Result => "konto.jahresergebnis",
            Account::Goodwill => "konto.firmenwert",
        }
    }
}

impl CostType {
    pub const ALL: [CostType; 17] = [
        CostType::Revenue,
        CostType::InventoryChange,
        CostType::Material,
        CostType::Personnel,
        CostType::Energy,
        CostType::Transport,
        CostType::Customs,
        CostType::Taxes,
        CostType::Marketing,
        CostType::Research,
        CostType::Interest,
        CostType::Depreciation,
        CostType::Maintenance,
        CostType::Overhead,
        CostType::Rent,
        CostType::Licenses,
        CostType::Other,
    ];

    pub fn text_key(self) -> &'static str {
        match self {
            CostType::Revenue => "kostenart.umsatz",
            CostType::InventoryChange => "kostenart.bestandsveraenderung",
            CostType::Material => "kostenart.material",
            CostType::Personnel => "kostenart.personal",
            CostType::Energy => "kostenart.energie",
            CostType::Transport => "kostenart.transport",
            CostType::Customs => "kostenart.zoelle",
            CostType::Taxes => "kostenart.steuern",
            CostType::Marketing => "kostenart.marketing",
            CostType::Research => "kostenart.forschung",
            CostType::Interest => "kostenart.zinsen",
            CostType::Depreciation => "kostenart.abschreibungen",
            CostType::Maintenance => "kostenart.instandhaltung",
            CostType::Overhead => "kostenart.gemeinkosten",
            CostType::Rent => "kostenart.pacht",
            CostType::Licenses => "kostenart.lizenzen",
            CostType::Other => "kostenart.sonstiges",
        }
    }
}

/// All text keys of reports; checked against the texts like the message keys.
pub fn text_keys() -> Vec<&'static str> {
    Account::ALL
        .iter()
        .map(|a| a.text_key())
        .chain(CostType::ALL.iter().map(|c| c.text_key()))
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct BalanceSheet {
    pub assets: Vec<(Account, Money)>,
    pub claims: Vec<(Account, Money)>,
    pub total: Money,
}

pub fn balance_sheet(ledger: &Ledger) -> BalanceSheet {
    let side = |assets: bool| {
        Account::ALL
            .iter()
            .filter(|a| a.is_asset() == assets)
            .map(|&a| (a, ledger.balance(a)))
            .collect()
    };
    BalanceSheet {
        assets: side(true),
        claims: side(false),
        total: ledger.total_assets(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IncomeStatement {
    /// Income and expenses by cost type (expenses negative), in a fixed order.
    pub lines: Vec<(CostType, Money)>,
    pub result: Money,
    pub cash_flow: CashFlow,
}

pub fn income_statement(period: &PeriodResult) -> IncomeStatement {
    let lines = CostType::ALL
        .iter()
        .filter_map(|t| period.by_type.get(t).map(|&m| (*t, m)))
        .collect();
    IncomeStatement {
        lines,
        result: period.total(),
        cash_flow: period.cash_flow,
    }
}
