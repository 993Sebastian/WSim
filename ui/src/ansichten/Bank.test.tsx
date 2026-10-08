import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { Bank, Befehl, Kern, Uebersicht } from "../kern";
import { vorschauKern } from "../kern/vorschau";
import { BankAnsicht } from "./Bank";
import { Befehle, useBefehl } from "./gemeinsam";

function Huelle({ kern }: { kern: Kern }) {
  const { senden, meldung } = useBefehl(
    kern,
    () => {},
    () => {},
  );
  return (
    <Befehle senden={senden} meldung={meldung}>
      <BankAnsicht kern={kern} stand="1915-01-01" />
    </Befehle>
  );
}

const EINE: Bank["banks"][number] = {
  company: 0,
  name: "Hausbank AG",
  own: true,
  deposit_spread: -0.01,
  deposit_rate: 0.02,
  loan_discount: 0.1,
  max_debt_ratio: 0.5,
  deposits_usd: 2_500_000,
  deposit_target_usd: 5_000_000,
  capacity_usd: 10_000_000,
  equity_usd: 1_000_000,
  cash_usd: 3_000_000,
  reserve_usd: 250_000,
  room_usd: 2_750_000,
  loans_given_usd: 500_000,
  loans: [
    { borrower: "Wendt GmbH", balance_usd: 500_000, rate: 0.045, start: "1914-03-01", months: 60 },
  ],
  interest_year_usd: 12_000,
  write_offs_year_usd: 0,
  result_year_usd: 8_000,
};

function zeige(aendern: (d: Bank) => void = () => {}) {
  const gesendet: Befehl[] = [];
  const vorschau = vorschauKern(0);
  const kern: Kern = {
    ...vorschau,
    bank: async () => {
      const d: Bank = { enabled: true, base_rate: 0.03, banks: [], subsidiaries: true };
      aendern(d);
      return d;
    },
    befehl: async (b) => {
      gesendet.push(b);
      return {} as Uebersicht;
    },
  };
  render(<Huelle kern={kern} />);
  return gesendet;
}

describe("Bank", () => {
  afterEach(cleanup);

  it("erklärt, wie man zu einer Bank kommt", async () => {
    zeige();
    expect(await screen.findByText(/Du hast keine Bank/)).toBeTruthy();
  });

  it("zeigt Einlagen und Kredite und legt die Bedingungen fest", async () => {
    const gesendet = zeige((d) => {
      d.banks = [structuredClone(EINE)];
    });
    const kredite = await screen.findByRole("table", { name: "Kredite an Firmen" });
    expect(within(kredite).getByText("Wendt GmbH")).toBeTruthy();
    expect(screen.getByText(/^Einlagen 2,5 Mio\./)).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Nachlass auf den Zins der Marktbanken"), {
      target: { value: "20" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Festlegen" }));
    await screen.findByText("Erledigt.");
    expect(gesendet).toEqual([
      {
        SetBank: {
          company: 0,
          settings: { deposit_spread: -0.01, loan_discount: 0.2, max_debt_ratio: 0.5 },
        },
      },
    ]);
  });
});
