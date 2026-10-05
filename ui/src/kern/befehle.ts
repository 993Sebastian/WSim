// Decisions of the player as the core reads them (crates/wsim-core/src/command.rs):
// content by key, amounts as Money units (hundredths of a cent).

/** USD → Money units of the core. */
export const geld = (usd: number): number => Math.round(usd * 10_000);

export type Preisart = { Market: { markup: number; floor: number } } | { Fixed: number };

export type Befehl =
  | { RenameCompany: { name: string } }
  | { FoundSite: { country: string; kind: string } }
  | { BuildFacility: { site: number; facility: string; count: number } }
  | { DevelopDeposit: { site: number; deposit: string } }
  | { SetProduction: { site: number; slot: number; recipe: string | null; utilization: number } }
  | { SetAutomation: { site: number; slot: number; level: number } }
  | { TakeLoan: { amount: number; years: number } }
  | { RepayLoan: { loan: number; amount: number } }
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
  | { SetAdvertising: { country: string; group: string; budget: number } }
  | { SetWagePremium: { site: number; premium: number } }
  | { SetPrice: { site: number; product: string; price: number } };
