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
| Forschung, Forschungsmodell | `research`, `ResearchModel` |
| Forschungsaufwand, Vorgriff, Nachzügler | `Effort`, `ahead_base`, `latecomer_discount` |
| Gemeingut | `public_domain_years` |
| Erfindung (im Spiel) | `inventions` |
| Eigenstrom, Einspeisevergütung | `own_electricity`, `feed_in_share` |
| Ergänzungsgut, netzabhängig | `ConsumptionType::Complement`, `needs_grid` |
| Startform, Startausstattung | `StartForm`, `StartSetup` |
| ruhender Markt | `Market::idle_since`, `settle_idle` |
| KI-Firma, KI-Modell | `CompanyKind::Ai`, `AiModel` |
| Kompetenz, Aggressivität | `competence`, `aggressiveness` |
| Schwierigkeit | `Difficulty` |
| Marktmaßstab | `market_scale` |
| Konzession | `Concession` |
| Anzahl (gleichartiger Anlagen) | `Slot::count` |
| Startbesetzung | `population::populate` |
| reale Firma, Namensgruppe | `RealCompany`, `NameGroup` |
| KI-Entscheidungen | `ai::decide` |
| Neugründung, Engpass | `found_companies`, `opportunity` |
| Preisuntergrenze, Höchstfaktor | `floor`, `price_max_factor` |
| Sicht (für die Oberfläche) | `views` (`Overview`, `RoundReportView`, `NewGameOptions`) |
| Spielsitzung, Spielstand | `wsim_session::Session`, `SaveEntry` |
| Beispielsichten (Browser-Vorschau) | `beispiel.json`, `vorschauKern` |
| Weltereignis, historisches Ereignis | `MessageKind::WorldEvent`, `HistoricalEvent` |
| Periodenergebnis, Vorrunde | `PeriodView`, `previous` |
| Autospeicherung | `AUTOSAVE_NAME` |
| Weltkarte, Kartenebene | `WorldMap`, `Ebene` (UI) |
| Länderdetail | `CountryDetail` |
| Engpass, Ursache | `Slot::limit` (`Limit`), `Cause` |
| Produktion, Markt, Forschung, Finanzen (Sichten) | `ProductionView`, `MarketView`, `ResearchOverview`, `FinanceView` |
| Berichte (Meldungsarchiv) | `BerichteAnsicht` (UI) |
| Lagerstätte erschließen, freie Lagerstätte | `DevelopDeposit`, `DepositOption` |
| Forschungslabor (Anlage im Forschungszentrum) | `LabView` |
| Absatz/Einkauf Vormonat | `sold_last_month`, `bought_last_month` |
| Eigentümer, Anteil, Mehrheit | `Holder` (Spieler, Firma, Privatbesitz), `Stake`, `majority_holder` |
| sehr komplexes Produkt (5–6 Ebenen) | `sehr_komplex` (Daten), `very_complex` |
| Tastaturkürzel | `TASTEN` (UI, `Tastenhilfe.tsx`) |

| Marke, Bekanntheit, Warengruppe | `Brand`, `awareness`, `GoodsGroupId` |
| Werbung, Werbebudget, Werbemittel | `Advertising`, `SetAdvertising`, `AdvertisingMedium` |
| Markenmodell (in `marktmodell.marke`) | `BrandModel`, `brand::month_start` |
| Mundpropaganda, Vergessen | `word_of_mouth`, `forgetting_per_month` |
| Präsenz (Anbieterwahl) | `presence`, `production_rate` |
| Marktdeckung (Startbesetzung) | `AiStart::market_cover` |
| Werbeanteil (KI) | `AiBehavior::advertising_share`, `ai::advertise` |
| Händlerlager zum Start | `population::stock_traders` |
| Preisfaktor, Preisniveau-Anteil | `market::price_factor`, `MarketModel::price_level_share` |
| Arbeitsproduktivität | `CountryState::labor_productivity` (`produktivitaet`) |
| Gemeinkosten (Verwaltung und Vertrieb) | `CostType::Overhead`, `ProductionModel::overhead_share`, `overhead_per_run_usd` |
| Wertschöpfung (zu Richtpreisen) | value added (`overhead_per_run_usd`) |
| je Produktart (Datentabelle) | `RawPerKind`, `[f64; 5]` mit `ProductKind::index` |
| Richtpreis-Marge, Plausibilitätsprüfung | `ProductionModel::reference_margin`, `health::reference_margin`, `check_reference_margins` |
| Vollkosten je Stück | `health::UnitCost`, `health::unit_cost` |
| Marktgesundheit (Versorgung, Engpässe, Preise) | `health::ProductHealth`, `health::last_month` |
| Normalauslastung (freie Anlagen) | `MarketModel::normal_utilization` (`auslastung_normal`) |
| Lagerziel, Ausgleich (KI-Auslastung) | `AiBehavior::stock_target_days`, `stock_adjust_days` |
