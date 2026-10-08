// Decisions of the player as the core reads them (crates/wsim-core/src/command.rs):
// content by key, amounts as Money units (hundredths of a cent).
import type { Auftrag, Geltung, Vorgabe, Vorgabefeld } from "./typen";

/** USD → Money units of the core. */
export const geld = (usd: number): number => Math.round(usd * 10_000);

export type Preisart = { Market: { markup: number; floor: number } } | { Fixed: number };

/** The pace of a start-up as the core names it (SU2). */
export type Tempo = "Normal" | "Fast" | "Thorough";

/** Start forms in commands (PE3), by their keys in views and texts. */
export type Startform = "Workshop" | "Trading" | "Investor" | "Bank";
export const STARTFORMEN: Record<string, Startform> = {
  werkstatt: "Workshop",
  handel: "Trading",
  investor: "Investor",
  bank: "Bank",
};

/** Levels of lifestyle in commands (PE3), by their keys in views and texts. */
export type Stufe = "Modest" | "Middle" | "Upscale" | "Luxury";
export const STUFEN: Record<string, Stufe> = {
  bescheiden: "Modest",
  buergerlich: "Middle",
  gehoben: "Upscale",
  luxurioes: "Luxury",
};

export type Befehl =
  | { RenameCompany: { name: string } }
  | { FoundSite: { country: string; kind: string } }
  | { FoundSiteOnPlot: { plot: number; kind: string; lease: boolean } }
  | { BuyPlot: { site: number } }
  | { BuildFacility: { site: number; facility: string; count: number; size?: string } }
  | { DevelopDeposit: { site: number; deposit: string } }
  | { SetProduction: { site: number; slot: number; recipe: string | null; utilization: number } }
  | { SetAutomation: { site: number; slot: number; level: number } }
  | {
      ProposeContract: {
        seller: number;
        buyer: number;
        product: string;
        per_month: number;
        price: number;
        months: number;
        min_quality: number;
        penalty: number;
      };
    }
  | { AnswerContract: { contract: number; accept: boolean } }
  | { CancelContract: { contract: number } }
  | { BuyVehicles: { vehicle: string; count: number } }
  | { SellVehicles: { vehicle: string; count: number } }
  | { SetLogistics: { mode: "Market" | "State" | "Fleet"; carry_for_others: boolean } }
  | {
      FoundSubsidiary: {
        name: string;
        country: string;
        capital: number;
        focus: "Production" | "Logistics" | "Investment" | "Bank" | "Investment" | "Bank";
      };
    }
  | { MoveCapital: { company: number; amount: number } }
  | { TransferSite: { site: number; to: number } }
  | {
      SetSubsidiaryFocus: {
        company: number;
        focus: "Production" | "Logistics" | "Investment" | "Bank";
      };
    }
  | { GoPublic: { share: number } }
  | { IssueBond: { amount: number; years: number } }
  | { RedeemBond: { bond: number } }
  | { IssueShares: { share: number } }
  | { SetDividend: { payout: number } }
  | { SetDividendPolicy: { policy: { Share: number } | { Amount: number } } }
  | { SpecialDividend: { amount: number } }
  | { BuyShares: { company: number; share: number } }
  | { SellShares: { company: number; share: number } }
  | { TakeOver: { company: number } }
  | {
      SetBank: {
        company: number;
        settings: { deposit_spread: number; loan_discount: number; max_debt_ratio: number };
      };
    }
  | { BuyBackShares: { share: number } }
  | { TakeLoan: { amount: number; years: number } }
  | { RepayLoan: { loan: number; amount: number } }
  | {
      FoundCompany: {
        name: string;
        form: Startform;
        country: string;
        capital: number;
      };
    }
  | { SetPersonSalary: { amount: number } }
  | { SetLifestyle: { level: Stufe } }
  | { ContributeCapital: { company: number; amount: number } }
  | { LendToCompany: { company: number; amount: number; rate: number; years: number } }
  | { WithdrawCapital: { company: number; amount: number } }
  | { SetSale: { site: number; product: string; mode: Preisart | null; keep: number } }
  | {
      SetPurchase: {
        site: number;
        product: string;
        target: number;
        max_price: number;
        min_quality: number;
      };
    }
  | { SetResearch: { site: number; technology: string | null } }
  | { SetDevelopment: { site: number; product: string | null } }
  | { SetAdvertising: { country: string; group: string; budget: number } }
  | { SetWagePremium: { site: number; premium: number } }
  | { SetPrice: { site: number; product: string; price: number } }
  | { MothballFacility: { site: number; slot: number; count: number } }
  | { RestartFacility: { site: number; slot: number } }
  | { SellFacility: { site: number; slot: number; count: number } }
  | { MakeOffer: { seller: number; object: Gegenstand; price: number } }
  | { AnswerOffer: { offer: number; answer: Antwort } }
  | { WithdrawOffer: { offer: number } }
  | { NameProduct: { product: string; name: string | null } }
  | { HireManager: { manager: number; position: Stellenangabe } }
  | { MoveManager: { manager: number; position: Stellenangabe } }
  | { DismissManager: { manager: number } }
  | { SetHiringByHead: { position: Stellenangabe; enabled: boolean } }
  | { RaiseSalary: { manager: number; salary: number } }
  | { SetTraining: { site: number; target: number | null } }
  | { PoachManager: { manager: number; position: Stellenangabe } }
  | { SetHeadquarters: { country: string; city?: string } }
  | { StaffDepartment: { department: string; staff: number } }
  | {
      SetParticipations: {
        budget: number | null;
        risk: number;
        limits: Record<string, number>;
      };
    }
  | { AnswerConcern: { concern: number; answer: Anliegenantwort } }
  | { InvestInVenture: { venture: number; amount: number } }
  | { GrantVenture: { venture: number; amount: number } }
  | { SellVentureStake: { venture: number; share: number } }
  | { OfferVentureStake: { venture: number; minimum: number | null } }
  | { BuyVentureStake: { venture: number; seller: number; price: number } }
  | { SteerVenture: { venture: number; pace: Tempo } }
  | { IntegrateVenture: { venture: number } }
  | { SpinOff: { site: number; sell: number } }
  | { SetBudget: { position: Stellenangabe; shares: [number, number] | null } }
  | { AskAgain: { position: Stellenangabe; topic: string } }
  | { SetStrategy: { scope: Geltung; field: Vorgabefeld; value: Vorgabe | null } }
  | { SetMandate: { mandate: Auftrag } }
  | {
      SetSalesPolicy: {
        buyer: "Traders" | "Companies";
        scope:
          | "Company"
          | { Country: string }
          | { Product: string }
          | { ProductInCountry: [string, string] };
        rule: { allowed: boolean; min_price: number | null; max_per_month: number | null } | null;
      };
    }
  | {
      SetBudgetRule: {
        kind: { level: { Site: string } | "Country" | "Continent" | "Board"; role: Rolle };
        scope: "Company" | { Continent: string } | { Country: string };
        shares: [number, number] | null;
      };
    };

/** The player's answer to a concern (MA2). */
export type Anliegenantwort = { Choose: number } | "Delegate" | "NeverAsk" | "Decline";

/** A role: the head of a unit or the specialist of a function. */
export type Rolle = "Head" | { Specialist: string };

/** A unit: a site by its number, a country or a continent by its key (MA3), the board (MA5). */
export type Einheit = { Site: number } | { Country: string } | { Continent: string } | "Board";

/** A position (MA1, MA3): the head of a unit or the specialist of a function there. */
export type Stellenangabe = { unit: Einheit; role: Rolle };

/** What an offer is for (M30): a site by its number, or a licence on a technology. */
export type Gegenstand = { Site: number } | { License: string } | { Area: string };

/** How the company whose turn it is answers an offer. */
export type Antwort = "Accept" | "Decline" | { Counter: { price: number } };
