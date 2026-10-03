# Glossar Daten ↔ Code

Datendateien und Texte sind deutsch, Code-Bezeichner englisch. Diese Liste ordnet
beides zu und wird mit jedem Meilenstein ergänzt.

| Daten / Spiel | Code |
| --- | --- |
| Simulationskern | core (`wsim-core`) |
| Katalog (unveränderliche Spielinhalte) | `Catalog` |
| Zustand (veränderlicher Spielstand) | `State` |
| Befehl | `Command` |
| Vorgabe | `Policy` |
| Stelle (Ebene einer Vorgabe) | `Scope` |
| Buchungssatz | `LedgerEntry` |
| Kostenart | `CostType` |
| Kostenstelle | `CostCenter` |
| Modifikator | `Modifier` |
| Meldung | `Message` |
| Land | `Country` |
| Produkt | `Product` |
| Rezept | `Recipe` |
| Anlage | `Facility` |
| Lagerstätte | `Deposit` |
| Technologie | `Technology` |
| Standort | `Site` |
| Firma | `Company` |
| Arbeitskräftegruppe | `LaborGroup` |
| Qualifikation | `Qualification` |
| Fachrichtung, Fachgebiet | `Specialization` |
| Einheit | `Unit` |
| Branche | `Branch` |
| Warengruppe | `GoodsGroup` |
| Transportklasse | `TransportClass` |
| Staatsmarkt | `StateMarketOffer` |
| Staatsnachfrage | `StateDemand` |
| Endkunden-Nachfrage | `ConsumerDemand` |
| Abbau (Rezept) | `extraction` |
| Nebenprodukt | `by_products` |
| Zeitreihe, Jahreswerte | `TimeSeries` |
| Geldbetrag | `Money` |
| Annäherung, Quelle | `Provenance` |
| Befund, Prüfbericht | `Finding`, `Report` |
| Spiel, Partie | `Game` |
| Spielzustand | `GameState` |
| Spieleinstellungen | `GameSettings` |
| Runde, Rundenlänge | `RoundLength` |
| Rundenbericht | `RoundReport` |
| Journal (Entscheidungen und Runden) | `JournalEntry` |
| Zufallsstrom | `SimRng`, `Stream` |
| Zustands-Hash | `StateHash` |
| Spielstand | `save`, `SaveHeader` |
| Befehl, Entscheidung | `Command` |
| Firma | `Company`, `CompanyId` |
| Ländermodell | `CountryModel`, `country_model` |
| Länderwerte (abgeleitet) | `CountryState` |
| Prägung | `CountryProfile` |
| Preisniveau | `price_level` |
| Einkommensfünftel | `income_quintiles_usd` |
| Erwerbspersonen | `labor_force` |
| Arbeitskräftepool | `labor_pool` |
| Produktionsmodell | `ProductionModel` |
| Standort | `Site`, `SiteId` |
| Anlage (gebaut, am Standort) | `Slot` |
| Charge (in Produktion) | `Batch` |
| Lagerbestand | `Stock` |
| Belegschaft | `workforce` |
| Buchführung, Konto | `Ledger`, `Account` |
| Kasse, Vorräte, Sachanlagen, Anlagen im Bau | `Cash`, `Inventory`, `FixedAssets`, `AssetsUnderConstruction` |
| Eigenkapital, Gewinnrücklagen, Jahresergebnis | `Equity`, `RetainedEarnings`, `Result` |
| Periodenergebnis | `PeriodResult` |
| Lagerstättenzustand | `DepositState` |
| Finanzmodell | `FinanceModel` |
| Kredit, Annuität, Rate | `Loan`, `instalment` |
| Kreditrahmen, Kreditlinie (Dispo) | `credit_limit`, `overdraft_limit` |
| Verlustvortrag | `loss_carryforward` |
| Zahlungsunfähigkeit | `is_insolvent`, `bankrupt` |
| Kapitalfluss | `CashFlow` |
| GuV, Bilanz | `IncomeStatement`, `BalanceSheet` |
| Markt, Marktpreis | `Market`, `market_price` |
| Marktmodell | `MarketModel` |
| Richtpreis | `reference_price` |
| Kaufschwelle, Kaufneigung | `purchase_threshold`, `propensity` |
| Besitzquote | `ownership` |
| Verkaufsangebot, Preismodus | `SaleOffer`, `PriceMode` |
| Einkaufsauftrag | `PurchaseOrder` |
| Handel (Menge, Umsatz) | `Trade` |
| Verkehrsmittel, Weg | `Vehicle`, `Way` |
| Transportmodell, Frachtdienst | `TransportModel`, `transport` |
| Route, Umweg, Umschlag | `Route`, `detour`, `handling` |
| Sendung, Empfänger | `Shipment`, `Consignee` |
| KI-Händler, Importlager, offene Nachfrage | `trade`, `Market::imports`, `open_demand` |
| Vorgabe, Verkaufsfreigabe, Geltungsbereich | `policy`, `SalesRule`, `Scope` |
| Abnehmergruppe | `BuyerGroup` |
