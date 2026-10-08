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
| Verlauf des Staatsbedarfs (Faktor je Jahr, M39) | `StateDemand::index` (`verlauf`), `StateDemand::per_million_gdp_at` |
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
| Verfügbarkeit eines Produkts (Nachfrage erst, wenn es sich herstellen lässt, M32) | `market::available` |
| Verfügbar seit (frühestes Jahr eines Rezepts), Verdrängung beim Staatsbedarf (M33) | `market::available_since`, `market::state_demand_left`, `MarketModel::state_displacement_years` (`verdraengung_staat_jahre`) |
| Einstieg in teure Märkte (M33) | `AiBehavior::entry_price_factor`, `entry_companies_max`, `entry_share` (`einstieg_preisfaktor`, `einstieg_firmen_max`, `einstieg_anteil`) |
| Lagerstätte für eine neue Konzession, Anlagen je Konzession (M33) | `ai::deposit_for`, `ai::units_for_concession`, `AiBehavior::reserve_years_min` (`vorrat_jahre_min`) |
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
| Förderindex, Höchstförderung im Jahr | `Product::output_index` (`foerderindex`), `Catalog::max_output` |
| Abbauwürdiger Vorrat im Jahr (wächst mit dem Förderindex) | `Catalog::reserve` |
| Spielstandspeicher (Dateien, Browser) | `wsim_session::SaveStore`, `DirStore`, `MemoryStore` |
| Browser-Version, Brücke zu JavaScript | Crate `wsim-web`, Modul `bridge`, `ui/src/kern/web.ts`, `kern.worker.ts` |
| Pacht und Förderabgaben | `Product::rent_share` (`pacht_anteil`), `CostType::Rent`, `production::rent_per_run_usd` |
| Höchste Änderung der Auslastung je Entscheidung | `AiBehavior::utilization_change_max` (`auslastung_aenderung_max`) |
| Knappheit nur bei Zahlungsbereitschaft (höchster Preis eines unbedienten Käufers) | `unmet_limit`, `note_unmet` in `market.rs` |
| Eigenstrom der KI | `ai::own_power` |
| Zahlungsbereitschaft (Einkauf der KI) | `willing` in `ai::operate` |
| Wiederbeschaffungspreis der Händler | `trade::Plan::replacement`, `import_floor` |
| Preisinsel, Arbitrage der Händler | `trade::plan` (`Destination::{arbitrage, arbitrage_below}`), `MarketModel::{arbitrage_gap, arbitrage_share}` (`haendler.arbitrage`; C1) |
| Förderkurve, Ausbau an mehreren Standorten | `production::deposit_output`, `ProductionModel::decline_from` (`foerderkurve_ab`); `ai::expand_at`, `AiBehavior::expansions_max` (`ausbau_je_pruefung_max`; C2) |
| Ausbausperre bei knappem Vorprodukt | `AiBehavior::expand_input_price_max` (`ausbau_vorprodukt_preis_max`) |
| Marktdeckung je Produktart | `AiStart::market_cover` (`[f64; 5]`) |
| nur als Nebenprodukt hergestellt | `health::made_as_main` |
| Entsorgung überschüssiger Nebenprodukte | `ProductionModel::by_product_stock_days` (`nebenprodukte_lager_tage`) |
| Lohnaufschlag (über die Landeslöhne) | `Site::wage_premium`, `SetWagePremium`, `ProductionModel::wage_premium_max` (`lohnaufschlag_max`) |
| Abwerben (Besetzung nach Lohnaufschlag) | `production::staff_sites` |
| Personalbedarf eines Standorts | `production::needed_workers`, `StaffLine` (Sicht) |
| Einstieg in teure oder knappe Märkte (C3) | `ai::newcomers`, `AiBehavior::entries_per_quarter` (`einstiege_je_quartal`) |
| Auslastung des Markts als Ausbau-Bremse (C4) | `ai::market_load`, `AiBehavior::expand_market_load` (`ausbau_markt_auslastung`) |
| Stadt (W2) | `catalog::City` (`Country::cities`, Daten `staedte`), Text `stadt.<ISO>.<id>` |
| Stadt des Hauptsitzes, Akademiker der Zentrale, Bürokosten (W2) | `Company::{hq_city, departments_staffed}`, `HqCityModel` (`zentrale.stadt`), `central::{city_in, hq_city, city_population, city_academics, office_per_employee, refresh_staffing, staffed, city_wanted, move_terms, ai_city}`; Befehl `SetHeadquarters { country, city }`, Fehler `UnknownCity` |
| Schulung je Standort: Niveau, Ziel, Kosten, Wirkung (W1) | `Site::{training, training_target}`, Befehl `SetTraining`, `ProductionModel::training` (`TrainingModel`, `produktionsmodell.schulung`), Modul `training` (`target`, `labor_factor`, `quality`, `daily_cost`, `month_start`, `ai_month_start`) |
| Schulungsziel der KI | `AiBehavior::training` (`verhalten.schulung`) |
| Lohnaufschlag der KI | `AiBehavior::wage_premium_step`, `wage_premium_max` (`lohnaufschlag_schritt`, `lohnaufschlag_max`), `ai::next_wage_premium` |
| Preis setzen (auch bei automatischem Preis) | `Command::SetPrice`, `CommandError::NoOffer` |
| Stückkosten eines Standorts (Material, Personal, Energie, Gemeinkosten, Pacht, Anlage) | `production::UnitCost`, `production::unit_costs`, `UnitCostView` |
| Ergebnis je Kostenstelle, interne Verrechnung | `PeriodResult::by_center`, `PeriodResult::site_type`, `Ledger::allocate` |
| Ergebnis des Standorts im Vormonat, Rohertrag je Produkt | `SiteResult`, `ProductResult` |
| Zu erledigen (Hinweise der Übersicht) | `views::hints`, `HintView`, `keys::HINT_*` (`hinweis.*`) |
| Verlauf je Monat (Kasse, Umsatz, Ergebnis) | `views::history`, `MonthView` |
| Ergebnis je Standort und Produkt (Finanzen) | `CenterResults`, `SiteLine`, `ProductResult` |
| Was lief (Rundenbericht) | `RoundReportView::products`, `round_products` |
| Produktmarkt, Anbieter, Wer kauft? | `views::product_market`, `ProductMarketView`, `SellerLine` |
| Absatzchancen (Weltkarte), Weltmarkt eines Produkts | `views::world_market`, `WorldMarketView` |
| Chancen eines Markts (Mangel, teuer, wenige Anbieter) | `MarketLine::chances`, `views::play::chances` |
| Versorgung (Anteil bedienter Nachfrage von Verbrauchern und Staat) | `MarketLine::supply` |
| Spielstand als Datei (Browser-Version) | `Kern::spielstandDatei`, `spielstandEinlesen` (UI), Worker-Anfragen `datei`, `einlesen` |
| Technologiebaum, Stand einer Technologie (bekannt, in Arbeit, erforschbar, gesperrt) | `TechnologyView::status`, `leads_to` (Führt zu) |
| Schätzung mit einem Labor (Dauer, Kosten) | `LabEstimate`, `views::play::lab_estimate`, `TechnologyView::one_lab` |
| Schaltet frei: Anlagen, Verfahren, Produkte | `FacilityUnlock`, `RecipeUnlock`, `TechnologyView::products` |
| Menge mit drei gültigen Stellen (UI) | `formatMenge` |
| Geführte Einführung, Weg je Startform, Schritt | `Einfuehrung`, `PFADE`, `Pfad`, `Schritt` (UI) |
| Markierung für die Einführung | Attribut `data-tour` (UI) |
| Lagerveränderung im Rundenbericht | `PeriodView::inventory_change_usd` |
| Währung, Kurzzeichen, Kurs je US-Dollar | `currency::Currency` (`key`, `symbol`, `rate`), `Rate::Points` (`kurse`) |
| Bindung an eine andere Währung | `Rate::Peg` (`bindung`: `an`, `faktor`) |
| Leitwährung, Basisjahr, Teuerung danach | `CurrencyModel::lead`, `base_year`, `inflation_after` (`leitwaehrung`, `basisjahr`, `teuerung_danach`) |
| Preisindex der USA | `CurrencyModel::us_prices` (`preisindex.werte`), `us_prices_at`, `inflation_since_base` |
| Währungszeiträume eines Landes | `CurrencyModel::periods` (`landeswaehrungen.perioden`), `currency_at` |
| Geldanzeige (Währung und Faktor), Anzeigeoptionen | `MoneyDisplay`, `MoneyOptions` (`Overview::money`); UI: `Geldanzeige`, `Geldoptionen`, `setzeGeldanzeige` |
| Kaufkraft 2026 / Preise der Zeit | `MoneyOptions::home_base`, `lead_base` / `home_then`, `lead_then`; UI: `GeldWahl.preise` (`basis`, `zeit`) |
| Betrag in der gezeigten Währung, zurück in Spieldollar (UI) | `inAnzeige`, `ausAnzeige`, `geldFeld`, `geldEinheit` |
| Zahlenfeld mit Tausenderpunkten beim Tippen; ganze Zahl, negativ, mit Tausenderpunkten (UI) | `ZahlFeld`, `ZahlEingabe`, `zahlEingeben`, `zahlGlaetten`; `Zahlart` (`ganzzahlig`, `negativ`, `gruppieren`) |
| Deutsch getippte Zahl lesen, in Teile zerlegen (UI) | `zahlLesen`, `zahlTeile` (`Zahlteile`) |
| Währungen eines Landes im Länderdetail, Kurs der Zeit | `CountryDetail::currencies` (`CurrencyPeriodView`), `currency_per_usd`, `CurrencyModel::periods_of` |
| Stilllegen, wieder anfahren, verkaufen (Anlage) | `Command::MothballFacility`, `RestartFacility`, `SellFacility` (M22) |
| Betriebszustand einer Anlage (läuft, stillgelegt, fährt wieder an) | `state::Operation` (`Running`, `Mothballed`, `Restarting`), `Slot::operating` |
| Restbuchwert, Verkaufserlös, Schrottwert | `Slot::book_value`, `production::sale_value`, `ProductionModel::scrap_share` (`verkauf.schrottwert`) |
| Überkapazität abbauen, wieder anfahren (KI) | `ai::retire`, `ai::restart_where_short`, `ai::output_and_offtake`, `AiBehavior::mothball_utilization`, `mothball_target_utilization`, `mothball_price_max`, `restart_utilization`, `sell_after_months` |
| Aufholen knapper Preise unter dem Richtpreis | `MarketModel::catch_up_max` (`preisanpassung.aufholen_max`), `market::adjust_price` |
| Untergrenze eines Nebenprodukts nach dem Brennwert (KI) | `ai::fuel_value` (`heizwert_mwh`) |
| Gelegenheit für eine Firma mit ihren Verfahren | `ai::opportunity` (`builder`) |
| Engpass-Suche mit Ausweichen, Standortwahl nach Fracht (M32) | `Chain::bottleneck`, `Chain::site_for` |
| Pionier (Produkt, das noch niemand herstellt; M32) | `ai::pioneer` |
| Marktlücke (Forschung, M32) | `gap_technologies`, `research_gap_companies` (`forschung_luecke_firmen`) |
| Vorlauf für kommende Produkte (M39) | `AiBehavior::research_lead_years` (`forschung_vorlauf_jahre`), `market::available_at_start` |
| Etappe (Etappenziel), Art, Wert | `catalog::Milestone`, `MilestoneCondition` (`etappen`: `art`, `wert`), `MilestoneId` |
| Erreichte Etappen des Spielers, Fortschritt | `GameState::milestones`, `milestones::check`, `milestones::progress`; Sicht `MilestoneView` (`Overview::milestones`); UI: `Etappen`, `ETAPPEN_SPEICHER` |
| Monatsreihe eines Markts (Preis, Absatz, eigener Absatz) | `state::MarketHistory` (`Market::history`), `competition::record_history`, `marktmodell.verlauf_monate`; Sicht `MarketMonthView` (`ProductMarketView::history`); UI: `MarktVerlauf`, `Marktmonat` |
| Wettbewerbsmeldungen (neuer Anbieter, Anbieter weg, Preissenkung) | `competition::competitor_news`, `state::WatchedMarket` (`GameState::watched_markets`), `marktmodell.meldung_preissenkung`; Schlüssel `meldung.ki.anbieter_neu`, `anbieter_weg`, `preissenkung` |
| Produktionsketten, Spitze einer Kette | `views::chains` (`ChainsView::roots`, `ChainProduct`, `ChainRecipe`), Sitzung `chains`, Befehl `ketten`; UI: `KettenAnsicht`, `Ketten`, `KettenProdukt` |
| Weiterlaufen bis Jahresende / zur nächsten Warnung | `Session::end_round_until` (`runde`, `jahresende`, `meldung`), `RoundReportView::rounds`, `stop`; Befehl `runde_beenden` mit `bis`; UI: `Weiterlaufen`, `MEHRERE` |
| Erklärung eines Werts (Teile von Preis, Nachfrage, Stückkosten) | `PriceParts`, `DemandParts`, `FifthParts` (`ProductMarketView::price_parts`, `demand_parts`), `market::successors`, `market::displaced`; UI: `Erklaerung`, `PreisTeile`, `NachfrageTeile`, `KostenTeile` |
| Währungszeitraum, gesetzlicher Umstellungskurs | `currency::Period` (`from`, `currency`, `conversion` ← `perioden[].umrechnung`) |
| Währungsumstellung (Meldung) | `currency::Reform`, `CurrencyModel::reform_at`, `game::currency_reforms`; Schlüssel `meldung.waehrungsreform`, Art `ereignisart.waehrung` |
| Auf gültige Stellen runden | `math::round_significant` |
| Ereignisart Wirtschaftspolitik | `reform` (`build::ai::EVENT_KINDS`) |
| Rang (Platz unter allen Firmen) nach Eigenkapital und Umsatz | `ranking::standing`, `ranking::Standing` (`GameState::standings`), `ranking::equity`, `revenue_of_year`, `year_before`, `month_end`; Sicht `RankView`, `StandingView` (`Overview::rank`); Schlüssel `meldung.rang`, `meldung.rang_vorjahr`; UI: `RangAnzeige`, `Rang`, `Platzierung` |
| Kaufangebot, Gegenangebot, Annahme, Ablehnung, Rücknahme | `deals::Offer` (`GameState::offers`, `next_offer`), `OfferStatus`, `OfferAnswer`; Befehle `MakeOffer`, `AnswerOffer`, `WithdrawOffer` (M30) |
| Gegenstand eines Angebots: Standort, Lizenz, Bereich | `deals::DealObject` (`Site`, `License`, `Area`) |
| Bereich (alle Standorte einer Warengruppe mit Marke) | `DealObject::Area`, `deals::area_sites`, `deals::AreaValue`, `deals::area_value`; Sicht `AreaView` (M31) |
| Markenwert (Werbung für dieselbe Bekanntheit) | `deals::brand_value` |
| Neubaupreis eines Standorts | `deals::new_site_cost` |
| Selbst gebraucht (Kraftwerk, einziges Labor) | `deals::needed_by_owner` |
| Grundwert, Buchwert, Ertragswert, Restwert eines Standorts | `deals::SiteValue` (`base`, `book`, `earnings_value`, `liquidation`), `deals::site_value`; Sicht `SiteValueView` |
| Höchstpreis und Aufschläge eines Käufers (Wettbewerb, Belegschaft, eigenes Geschäft, Neubau) | `deals::Advantages`, `deals::advantages` |
| Lizenzwert (ersparte Forschungskosten) | `deals::license_value` |
| Firmenwert eines gekauften Standorts | `state::Goodwill` (`Site::goodwill`), `Account::Goodwill`, `Site::acquired` |
| Kostenart Lizenzen | `CostType::Licenses` |
| Kaufmodell (Parameter) | `catalog::DealModel`, `DealAi` (`parameter/kaufmodell.yaml`) |
| KI kauft und verkauft | `deals::ai_offers`, `deals::simulate_day` (Antworten, Verfall), `best_deal` |
| Ansicht Wettbewerb (Angebote, Firmen) | Sichten `OffersView`, `CompaniesView`, `CompanyDetailView` (`offers`, `companies`, `company_detail`); Befehle `angebote`, `firmen`, `firma`; UI: `WettbewerbAnsicht`, `BereichKarte` (Typ `Geschaeftsbereich`) |
| Region, umfasste Länder (`umfasst`), Teilland (`teilland.<ISO>`) | `Country::members`; Sicht `CountryDetail::members`; Prüfung `countries::check_regions` (M34) |
| Früherer Schlüssel eines Eintrags (Spielstände) | `KeyTable::add_alias`, `ids::take_last_rank` (Vorrang in `PerId`) |
| Firmenzahlen der KI-Regeln mit der Zahl der KI-Firmen | `ai::per_companies` |
| Anbieter und Käufer eines Produkts je Land (Markträumung) | `market::Traders` |
| Grundstücksmodell (Parameter), Größenklasse, Lage (Stadt, Hafen, Land) | `catalog::PlotModel`, `PlotClass`, `Location` (`City`, `Port`, `Rural`), `LocationModel` (`parameter/grundstuecksmodell.yaml`; M35) |
| Fläche einer Anlage | `Facility::area_ha` (`anlagen[].flaeche_ha`) |
| Grundstück, Kauf, Pacht (Besitz) | `state::Plot`, `PlotId`, `Tenure` (`Owned`, `Leased`), `GameState::plots`, `Site::plot`; Konto `Account::Land`; Befehle `FoundSiteOnPlot`, `BuyPlot`; Modul `plots` |
| Gewerbefläche eines Landes, freie Grundstücke | Sichten `LandView`, `PlotView` (`land_view`), `SitePlotView`; UI: `GrundstueckWahl`, `useGewerbeflaeche`, `GrundstueckKarte`, `Gewerbeflaechen` |
| Anlagengröße (sehr klein bis sehr groß), Kapazitätsfaktor | `catalog::FacilitySize`, `SizeModel` (`ProductionModel::sizes`, `produktionsmodell.anlagengroessen`; M36) |
| Weiterentwicklung, Entwicklungsstufe, Wirkung je Stufe, Gemeingut einer Stufe | `development` (Modul), `DevelopmentModel` (`forschungsmodell.weiterentwicklung`), `Company::development` (`Development`: `levels`, `points`), `Site::development`, `GameState::developments`, `development::{level, public_level, effect, next_effort, basis, can_develop}`, `Effect`; Befehl `SetDevelopment` (M37) |
| Nutzen und Amortisation einer Stufe (KI) | `development_benefit_per_level`, `development_payback_years` (`kimodell.verhalten.entwicklung_*`), `ai::development_target` |
| Versteigerung der Standorte einer insolventen Firma, Mindestgebot, Zuschlag | `Company::auction_until`, `deals::in_auction`, `deals::auction_minimum`, `close_auctions`, `auction_site`; `DealModel::insolvency_days`, `insolvency_min_share` (`kaufmodell.insolvenz`; M38) |
| Produktname, Namensstil, Stamm, Muster, Hausmarke, ausgeschlossene Namen | `Company::product_names`; `catalog::ProductNaming` (`house_brand`, `excluded`, `styles`, `style_of_group`), `NamingStyle` (`stems`, `patterns`, `numbers`, `letters`, `additions`), `NamePattern` (`ki/produktnamen.yaml`; M42) |
| Namen bilden und prüfen, Vorschläge | Modul `product_names` (`generate`, `suggestions`, `taken`, `same_name`, `named_products`, `name_new_company`); `command::check_product_name`; Zufallsstrom `Stream::ProductName`; Befehl `NameProduct`; Fehler `NameError::Excluded`, `CommandError::NotNameable` |
| Produktname in Sichten | `SellerLine::product_name`, `OfferView::product_name`, `ProductMarketView::{nameable, own_name, name_suggestions}`, `CompanyDetailView::products` (`NamedProductView`); UI: `Produktname` (Markt) |
| Nachfolgemodell, Kennzeichen der Generation | `product_names::successor`, `NamingStyle::successors` (`nachfolger`); Meldung `DEVELOPMENT_SUCCESSOR` (B1) |
| Engpass ohne Ausweg (KI baut ein Produkt, das niemand herstellt, trotz knapper Vorprodukte) | `ai::Chain::bottleneck` mit `Chain::makers` (M41) |
| Angebote eines Produkts in einem Spielstand (Diagnose) | `wsim angebote <spielstand> <produkt>` (`show_offers` in `wsim-cli`) |
| Vorprodukt geplanter Anlagen (Startbesetzung: eine Anlage in der kleinsten passenden Größe) | `Need::input`, `smallest_size`, `Placement::size` in `population` |
| Entscheidung, Thema, Option (Art), Schritt (MA0) | `decision::Decision`, `Topic`, `Choice` (`ChoiceKind`), `Step` |
| Entscheider, Antwort (Regel, andere Option, nichts jetzt) | `decision::Decider`, `Verdict::{Rule, Choice, Hold}`; KI: `Rules`; Aufzeichnung: `Recorder` |
| Entscheidung vorlegen, bevor eine Regel handelt | `decision::decided`, in `ai`: `act`, `routine`, `overcapacity`, `research_decision`, `new_site_steps` |
| Bewertung einer Option (angerechneter Betrag, Wirkung je Jahr, einmalig) | `decision::assess` → `Assessment { amount, effect, once }`, `decision::amount` |
| KI-Entscheidungen mit eigenem Entscheider | `ai::decide_with` (`ai::decide` nutzt `Rules`) |
| Fingerabdruck eines Spielzustands | `game::hash_of` |
| Manager-System (Parameter): Bereich, Ebene, Fachstellen je Standorttyp, Pool, Fähigkeiten | `catalog::ManagementModel` (`functions`, `levels`, `specialists`, `head_discount`, `notice_base`, `salary_group`, `severance_months`, `pool`, `skills`), `ManagementFunction`, `ManagementLevel`, `ManagerPoolModel`, `SkillModel` (`parameter/management.yaml`; MA1) |
| Manager, Schwerpunkt, Fachkompetenz, Erkennen, Urteilsvermögen, Führung, Risikoneigung, Fragefreude, Eindruck | `state::Manager` (`focus`, `expertise`, `detection`, `judgment`, `leadership`, `risk`, `talkativeness`, `impression`), `ManagerId`, `GameState::managers`, `next_manager` |
| Stelle (Leitung, Fachstelle), Anstellung, Gehalt | `state::Position` (`site`, `role`), `Role::{Head, Specialist}`, `Job` (`company`, `position`, `salary`, `since`) |
| Stellen eines Standorts, Inhaber, Gehaltsforderung, Stärke, angezeigte Stufe | `management::{positions, holder, salary_demand, strength, shown_level, skill, skill_keys}` |
| Bewerberpool je Kontinent, Monatswechsel | `management::{month_start, pool_size}`; Zufallsstrom `Stream::ManagerMarket` |
| Gehälter, Abfindung | `management::month_end`, `dismiss` (Kostenart `Personnel`, Kostenstelle Standort) |
| Einstellen, Versetzen, Entlassen | Befehle `HireManager`, `MoveManager`, `DismissManager`; Fehler `UnknownManager`, `ManagerEmployed`, `NotYourManager`, `UnknownPosition`, `PositionTaken` |
| Routine der Standortstellen, Prüftermin, Bemerken | `management::simulate_day` mit dem Entscheider `Staff`; `ai::site_routine`; Zufallsstrom `Stream::Manager` |
| Stelle entfällt mit dem Standort | `management::release_site` (aus `deals::hand_over`) |
| Namensfolge (Familienname zuerst) | `NameGroup::surname_first` (`namensgruppen[].familienname_zuerst`) |
| Budget je Stelle (Anteil je Entscheidung und Jahr, Bezug, Sockel), verbraucht | `ManagementLevel::{budget_specialist, budget_head}`, `ManagementModel::budget_floor` (`budget_fach`, `budget_leitung`, `budget_sockel_gehaelter`); `management::{budget, budget_base, budget_shares, default_shares, spent}`; `PositionState` (`budget`, `spent`, `year`); Befehl `SetBudget` (MA2) |
| Routinethemen | `ManagementModel::routine_topics` (`routine_themen`) |
| Anliegen, Option eines Anliegens, Empfehlung, Frist | `state::Concern` (`recommended`, `options`, `deadline`), `ConcernOption` (`amount`, `forecast`, `once`), `GameState::concerns`, `next_concern`; Parameter `ConcernModel` (`anliegen`) |
| Grund eines Anliegens (Budget je Entscheidung, Jahresbudget, immer fragen, Kredit) | `state::ConcernReason::{Decision, Year, Always, Finance}` |
| Antwort: Option wählen, „Entscheide selbst“, „Nicht mehr fragen“, „Ablehnen“ | Befehl `AnswerConcern` mit `management::ConcernAnswer::{Choose, Delegate, NeverAsk, Decline}`; Ausgang `ConcernStatus::{Chosen, Delegated, Muted, Declined}` |
| Anliegen verfallen, anderweitig erledigt | `ConcernStatus::Expired`, `ConcernStatus::Settled` (`management::settle` nach jedem Befehl) |
| Stummgeschaltete und abgelehnte Themen, „Wieder fragen“ | `PositionState::{muted, blocked}`; Befehl `AskAgain` |
| Protokoll der Stelle („Was die Stelle selbst entschieden hat“) | `PositionState::log` (`PositionLog`) |
| Rückmeldung zur Wirkung | `state::Followup`, `GameState::followups`; Meldung `meldung.anliegen.folge` |
| Wichtiges Anliegen, Anhalten bei Anliegen (alle, wichtige, nie) | `management::important`; `Session::end_rounds` (`anhalten`), Halt `anliegen`; UI: `Anhalten`, `ANHALTEN_SPEICHER` |
| Thema kommt am Standorttyp vor | `management::arises` |
| Hinweis entfällt, weil eine Stelle den Bereich übernimmt | `management::covered` (in `views::hints`) |
| Sichten der Anliegen, Gruppe gleicher Anliegen, Schritt in Worten | `views::{concerns, ConcernsView, ConcernGroupView, ConcernView, ConcernOptionView}`; Textschlüssel `schritt.*`, `anliegen.begruendung.*`; UI: `AnliegenListe` (`Anliegen.tsx`) |
| Budget und Protokoll in der Organisation | `PositionView::{budget, log, quiet, open_concerns}` (`BudgetView`, `DecisionLogView`, `QuietTopicView`); UI: `StellenDetails` |
| Einheit (Standort, Land, Kontinent), Stelle einer Einheit | `state::Unit::{Site, Country, Continent}`, `Position { unit, role }` (`Position::at_site`, `site()`); `management::{units, has_unit, positions, level_of}` (MA3) |
| Fachstellen und Themen je Ebene | `ManagementLevel::{specialists, topics}` (`ebenen[].fachstellen`, `ebenen[].themen`); `management::arises` |
| Sitzland einer Einheit (Gehalt) | `management::seat_country` |
| Zuständigkeitskette, erste übernehmende Stelle | `management::{chain, first_taker, covered}` |
| Weiterleitung, Weg eines Anliegens, fragende Stelle | `state::Hop`, `Concern::path`, `management::asker`; Sichten `ConcernPositionView`, `HopView` |
| Nächste Leitung, Deckel | `management::superior`, `management::budget` |
| Budget-Vorgabe je Stellentyp, Geltungsbereich | `state::BudgetRule`, `PositionKind`, `UnitLevel`, `RuleScope`, `Company::budget_rules`; Befehl `SetBudgetRule`; `management::{rule_for, kind_of}`; Sichten `BudgetRuleView`, `PositionKindView`; UI: `Budgetvorgaben` |
| Strategisches Anliegen, Teil eines Anliegens | `Concern::parts`, `state::ConcernPart`, `management::bundle` (`anliegen.buendel_ab`, `ConcernModel::bundle_from`); Sicht `ConcernPartView` |
| Struktur und Werbung von Land und Kontinent | `ai::{unit_structure, unit_advertising}`; `management::{sites_cared_for, countries_cared_for}` |
| Strategievorgabe, Geltungsbereich (Firma, Kontinent, Land, Standort) | `strategy::{StrategySetting, StrategyScope}`, `Company::strategies`; Befehl `SetStrategy`; `strategy::{setting, effective, for_site}` (MA4) |
| Strategiefeld | `strategy::StrategyField::{Price, Stock, Wages, Supply, Investment, Reserve, Training}` (`preis`, `lager`, `personal`, `eigenfertigung`, `investition`, `reserve`, `schulung`) |
| Preisstrategie: Marktpreis, Premium, Kampfpreis, Mindestmarge | `strategy::PriceStrategy::{Market, Premium, Fight, MinMargin}`; `strategy::price_terms`; Daten `management.strategie` (`StrategyModel`) |
| Lager-Vorgabe (Reichweite der Vorprodukte, Lagerziel der Fertigwaren) | `strategy::StockStrategy { input_min_days, input_max_days, output_days }` |
| Lohnaufschlag-Spanne | `strategy::WageStrategy { min, max }` |
| Eigenfertigung oder Zukauf: eigene Ware zuerst, nach Preis, nur Zukauf | `strategy::SupplyStrategy::{OwnFirst, ByPrice, Buy}` (UI: `Bezugsweg`) |
| Investitionsbudget, Rest, bindendes Budget | `StrategyValue::Investment`, `strategy::{InvestmentBudget, investment_budgets, binding_budget, count_investment}` |
| Liquiditätsreserve (Monate laufender Kosten) | `StrategyValue::Reserve`, `strategy::{reserve_months, monthly_cost}` |
| Schulungsziel als Strategievorgabe (`schulung`) | `StrategyField::Training`, `StrategyValue::Training`, `strategy::training_for_site` (W1) |
| Anliegen wegen Reserve bzw. Investitionsbudget | `ConcernReason::{Reserve, Investment}` (`reserve`, `investition`); Sicht `ConcernView::{strategy_limit_usd, strategy_scope}` |
| Strategieansicht | `views::{strategy, StrategyView, StrategyUnitView, StrategyEntryView}`; UI: `StrategieAnsicht` (`Strategie.tsx`) |
| Verkaufsweg (Regel für KI-Händler oder andere Firmen) | `policy::{SalesPolicy, SalesRule, BuyerGroup, Scope}`; Befehl `SetSalesPolicy`; Sicht `SalesChannelView`; UI: `Verkaufswege` (`Strategie.tsx`) |
| Organisation je Einheit, Managermarkt je Einheit | Sichten `UnitOrgView`, `CountryOrgView::unit`, `ContinentOrgView::unit`, `views::{unit_key, unit_from_key}`; `Session::manager_market(einheit, stelle)`; UI: `EinheitKarte`, `einheitName`, `stellenangabe` |
| Vorstand, CEO, Ressort | `state::Unit::Board`, `UnitLevel::Board`, Leitung = CEO (`leitung.vorstand`), Fachstellen = Ressorts; `management::ceo_of`; Sicht `OrganisationView::board` (MA5) |
| Regelthemen (Kredite, Werbung: Stelle folgt der Regel) | `ManagementModel::rule_topics` (`regel_themen`) |
| Kasse, Kaufangebote und Antworten des Vorstands | `ai::manage_cash`, `deals::{board_offers, board_answers, best_deal_for}`, `decision::Topic::OfferAnswer` (`antwort`), `ChoiceKind::Counter` (`gegenangebot`) |
| Befugnis für Kredite, Verschuldungsgrenze | `management::lends`, `mandate::{over_debt, debt_share, loan_room}`; `ConcernReason::Debt` (`verschuldung`) |
| Strategieauftrag (Leitlinie, Ziele, Grenzen, Takt) | `mandate::{Mandate, Guideline, Goals, ReviewInterval}`, `Company::mandate`; Befehl `SetMandate`; `mandate::{aggressiveness, banned, blocked_object}`; Daten `management.strategieauftrag` (`MandateModel`) |
| Leitlinie: Wachstum, Ertrag, Sicherheit, Marktführerschaft | `Guideline::{Growth, Profit, Safety, Leadership}` (`wachstum`, `ertrag`, `sicherheit`, `marktfuehrung`) |
| Strategierücksprache, Bericht, Ziel, Chance, Risiko, Antrag | `review::{Review, Figures, GoalCheck, Goal, Chance, Risk, month_end, report, goals}`, `Company::reviews`; `ConcernReason::Proposal` (`antrag`); `ai::opportunities`; Sicht `views::{reviews, ReviewsView}`; UI: `RuecksprachenAnsicht` (`Ruecksprache.tsx`) |
| Umsatz je Produkt (Hauptbuch) | `ledger::PeriodResult::product_revenue` |
| Eindruck schärfen (Ressort Personal) | `management::{impression_share, shown_impression}`, `strategieauftrag.personal_schaerfe` |
| Lebendiger Managermarkt (Monatslauf) | Modul `staffing` (`month_start`, `simulate_day`); Daten `management.markt` (`ManagerMarketModel`) |
| Erfahrung, persönliche Obergrenze | `Manager::potential`, `staffing::set_potentials`, `rng::Stream::{ManagerMonth, ManagerPotential}`; `markt.erfahrung` |
| Zufriedenheit, Stufe (unzufrieden, gemischt, zufrieden), Marktwert | `Job::satisfaction`, `staffing::{satisfaction, satisfaction_target, satisfaction_level, market_value, unit_result}`; `SatisfactionModel` (`markt.zufriedenheit`) |
| Kündigung | `staffing::resignations`, `markt.kuendigung`; Meldung `meldung.manager.kuendigung` |
| Gehalt anpassen | Befehl `RaiseSalary`, `staffing::raise_salary` |
| Leitung stellt ein | Befehl `SetHiringByHead`, `PositionState::hires`, `staffing::{heads_hire, has_work}`; Sicht `PositionView::hires` |
| Abwerbung, Angebot an einen Manager, Sperrfrist | `state::PoachOffer`, `GameState::poach_offers`, `Manager::courted`; Befehl `PoachManager`; `staffing::{poach, poach_salary, courted_until}`; `PoachingModel` (`markt.abwerbung`) |
| Führung einer anderen Firma, freie Stelle, Abwerbeangebot, „über der Kasse“ | `views::organisation::{RivalManagerView, FreePositionView, PoachOptionView, PlaceView, rival_managers, free_positions}`, `CompanyDetailView::{managers, free_positions}`, `CandidateView::over_cash`, `ManagerMarketView::cash_usd`; Meldungen `MANAGER_POACH_WON`, `MANAGER_POACH_KEPT` (B1); UI: `Fuehrung`, `ManagerKarte` (Wettbewerb) |
| Gegenangebot, Gehen lassen | Befehle `MatchOffer`, `LetGo`; `ChoiceKind::{Counter, LetGo}` (`gegenangebot`, `gehen_lassen`); Thema `Topic::Poaching` (`abwerbung`); `ConcernReason::Poaching` |
| Hauptsitz, Umzug (Verlegen) | `Company::{headquarters, relocation}`, `state::Relocation`; Befehl `SetHeadquarters`; Modul `central` (`set_headquarters`, `relocation_cost`, `month_start`); Daten `zentrale.hauptsitz` (`CentralModel`, `HeadquartersModel`); Sicht `CentralView` (`OrganisationView::central`); UI: `Hauptsitz` (`Organisation.tsx`) |
| Zentralabteilung (Strategie, Finanzen, Personal, Recht, Marketing), Angestellte, Leitung | `catalog::{Department, DepartmentKind}`, `Company::departments`; Befehl `StaffDepartment`; `central::{staff, employees, head, head_position, workload, performance, strength, cases, monthly_cost, month_end}`; Sicht `DepartmentView` (`CentralView::departments`); UI: `Abteilungen`, `AbteilungZeile` (`Organisation.tsx`), Typ `Abteilung` |
| Kapazität, Güte, Abdeckung, Stärke einer Abteilung | `central::Performance { capacity, quality, coverage, strength }` |
| Genauigkeit (der Leitung) | `CentralModel::accuracy`, `central::accuracy`, `Responsible::accuracy` (`management.rs`) |
| beobachtete Länder (Strategie), geprüfte Technologien (Recht) | `central::{observed_countries, legal_technologies}`, `deals::{Search, Deal, deals_for, BoardPlan}` |
| Risikoaufschlag-Ersparnis (Finanzen), Schulung (Personal), Werbewirkung (Marketing) | `central::{premium_cut, experience_chance}`, `finance::{loan_rate, rate_for_debt}`, `brand::month_start` |
| Beteiligungen (Vorgabe): Budget im Jahr, Risikobereitschaft, Freigabegrenze | `state::Participations { budget, risk, limits, spent, year }`, `Company::participations`; Befehl `SetParticipations`; `central::{set_participations, participations_left, count_purchase, release_limit}`; Gründe `ConcernReason::{Participations, Limit}` (`beteiligung`, `freigabe`); Sicht `ParticipationsView`, `ReleaseLimitView`; UI: `Beteiligungen` (`Strategie.tsx`), Typ `Beteiligungen` |
| Ressort Strategie, Ressort Recht, Thema Lizenz | Bereiche `strategie`, `recht`; `Topic::License` (`lizenz`), `deals::{deal_topic, answer_topic}` |
| Umschulden | Befehl `RefinanceLoan`, `central::{refinance, refinance_rate, months_left, refinance_candidates}`, `CentralModel::refinance` (`RefinanceModel`), `Topic::Refinance`, `ChoiceKind::Refinance` |
| Gehaltsrunde | `Topic::SalaryRound`, `central::salary_round_candidates`, Befehl `RaiseSalary` |
| Empfehlung (einer Abteilung), Begründung | `management::recommend`; Begründungen `keys::BECAUSE_{REFINANCE, SALARY_ROUND, FIT, SPREAD, SHORTCUT}`, Schritte `STEP_{REFINANCE, RAISE}` (`views/concerns.rs`) |
| Schätzung (einer Leitung), Fehlgriff | `Decider::estimate`, `Staff::estimate`, `management::estimates` (Tests) |
| Trefferquote, Bewertung | `Manager::{judged, hits}`, `state::Judgment`, `GameState::judgments`; `central::{hit_rate, hit_factor, record_judgment}`, `CentralModel::hit_rate` (`HitRateModel`); Sicht `ManagerView::{judged, hit_rate}`, `DepartmentView::{head_judged, head_hit_rate}`; UI: `trefferquote` (`Organisation.tsx`) |
| Bewertung nach dem Erfolg (Übernahme, Start-up-Empfehlung) | `state::Appraisal::{Takeover, Venture}`, `Judgment::appraisal`; `central::{record_appraisal, takeover_appraisal, judged_hit}` (ZA4) |
| Zentrale der KI, Budget der Zentrale, Mindestlast | `catalog::CentralAiModel { revenue_share, order, seat_revenue_share, seat_gdp_share, payback_years, lock_years }` (`zentrale.ki`: `anteil_umsatz`, `reihenfolge`, `mindestlast`, `sitz`); `central::{ai_month_start, ai_plan, yearly_cost, year_figures}`; Leitung: `staffing::next_position`; CLI-Weltbericht „Zentralen der KI“ (ZA4) |
| Sitzverlegung der KI, Ersparnis, Sperre nach dem Umzug | `central::{ai_seat, seat_saving}`, `Company::relocated`; Meldung `meldung.hauptsitz.konkurrenz` (`keys::RIVAL_HEADQUARTERS`); Sicht `CompanyRowView::{departments, central_staff, moving_to, moving_until}`; UI-Typ `Firmenzeile` (ZA4) |
| Start-up, Erfinder und Gründungen, Wagniskapital | `state::Venture`, `GameState::{ventures, next_venture}`, Modul `ventures` (`month_start`, `found`, `advance`, `succeed`, `prune`); `catalog::{VentureModel, VenturePhase, Inventor}` (`startups`, `erfinder`); Sicht `VenturesView`, `VentureView`, `StakeView`; UI: `BeteiligungenAnsicht` (`Beteiligungen.tsx`), Typen `StartUps`, `StartUp`, `StartUpEigner` |
| Ziel: neue Technologie, Verbesserung (Stufe), Vorlauf | `state::VentureTarget::{Technology, Development}`, `Venture::lead`; `ventures::{new_technologies, improvements}` |
| Phase (Idee, Prototyp, Marktreife), Finanzierungsrunde, Bewertung | `Venture::{phase, capital, chance, raised, round_until, phase_until}`, `VenturePhase::{months, capital, chance, valuation}`; `ventures::{start_phase, issue}` |
| Investoren (außerhalb des Spiels), Gründer, Verwässerung | `Holder::{Investors, Private}`, `Venture::owners` (`Stake`) |
| Erfolg, gescheitert, kein Geld, überholt | `VentureStatus::{Active, Succeeded, Failed}`, `VentureFailure::{Phase, Funding, Overtaken}`; Meldungen `meldung.startup.{erfindung, weiterentwicklung}` |
| Erfolgschance, Einschätzung (gering, mittel, hoch), Unschärfe | `ventures::{phase_chance, success_chance, shown_chance, insight}`, `Venture::blur`, `VentureModel::{blur, chance_levels}` |
| Häufigkeit der Start-ups (keine, wenige, normal, viele) | `GameSettings::ventures`, `VentureModel::{frequencies, default_frequency, default_factor}`; `NewGameRequest::startups`, `NewGameOptions::{startups, default_startups}` (`FrequencyOption`); CLI `run --startups` |
| Beteiligung (an einem Start-up), Anteil, Buchwert, Portfolio | `Holder::Company`, `Venture::{owners, book}`, `ventures::{share_of, amount_of, companies_of}`; Konto `Account::Participations` (`konto.finanzanlagen`, Finanzanlagen), Kostenart `CostType::Investments` (`kostenart.beteiligungen`); `VenturesView::{holdings, portfolio_book_usd, portfolio_value_usd}`, `VentureView::{own_share, own_book_usd}` |
| Wert, Wert bei Erfolg, Bewertung vor/nach der Runde | `ventures::{value, success_value, expected_success_value, share_per_dollar}`, `VentureStakeModel::success_factor` (`erfolg_faktor`); `VentureView::{value_usd, success_value_usd}` |
| Zusage (in einer Runde), aufstocken (Anteile kaufen) | Befehl `InvestInVenture`, `Venture::pledges`, `ventures::{invest, pledge, close_round, close_round_with}`, `VentureStakeModel::buy_premium` (`kauf_aufschlag`); `VentureView::{own_pledge_usd, invest_mode, invest_max_usd}` (`runde`, `anteile`) |
| Fördergeld | Befehl `GrantVenture`, `Venture::grants`, `ventures::grant`, `VentureStakeModel::grant_effect` (`foerderung_wirkung`) |
| Verkauf (an Investoren), Abschlag | Befehl `SellVentureStake`, `ventures::sell`, `VentureStakeModel::sale_discount` (`verkauf_abschlag`); `VentureView::sale_value_usd` |
| Verkauf an Firmen (Bieterverfahren), Mindestpreis, Gebot, Zuschlag | Befehle `OfferVentureStake`, `BuyVentureStake`; `Venture::sales`; `ventures::{offer_stake, buy_stake, stake_bid, best_stake_bid, sale_blocked, settle_sales, kept_after_rounds}`; Fehler `CommandError::NoStakeOffer`; Meldungen `meldung.startup.{anteil_verkauft, anteil_nicht_verkauft, anteil_bestes_gebot}`; Sicht `VentureView::{own_value_usd, own_offer_usd}`, `VenturesView::{offers_settle, company_premium_max}`; UI: `AnFirmen` (`Beteiligungen.tsx`) (ZA4) |
| Sperrminorität, Mehrheit | `VentureStakeModel::{blocking, majority}` (`sperrminoritaet`, `mehrheit`), `ventures::{blocked, majority_company}`; `VentureView::{majority, blocked}` |
| Lenken, Tempo (normal, zügig, gründlich) | Befehl `SteerVenture`, `state::VenturePace::{Normal, Fast, Thorough}`, `Venture::pace`, `VentureStakeModel::{fast, thorough}` (`lenkung`); Texte `startup.lenkung.<id>`; UI-Typ `Tempo` |
| Eingliedern, Tochterfirma, Mutter | Befehl `IntegrateVenture`, `Venture::parent`, `ventures::{integrate, integration_price, keep_parent}`; `VentureView::{subsidiary, parent, integration_usd}` |
| Ausgang: Tochter, Auszahlung, Börsengang, neue KI-Firma | `state::VentureExit::{Parent, Listed}`, `Venture::exit`, `ventures::{settle_success, buy_out, to_parent, exit_stake}`, `ai::found_from_venture`; Meldungen `meldung.startup.{tochter, ausgezahlt, uebernommen, boersengang, neue_firma, rueckzahlung, verloren, forschungsbonus}` |
| Ausgründung, Projekt, Fortschritt, Restdauer, Konkurrenz am selben Ziel | Befehl `SpinOff`, `ventures::{spin_off, spin_off_plan, spin_off_value, SpinOffPlan, months_from, ahead_of_history, rivals}`, `SpinOffView::{lead, months, rivals}`, `Venture::origin`, `VentureStakeModel::spin_off_progress_min` (`ausgruendung_fortschritt_min`), `rng::Stream::SpinOff`; Fehler `CommandError::{NoSpinOff, SpinOffTooEarly}`; Sicht `SpinOffView`, `VenturesView::{spin_offs, spin_off_min}`, `VentureView::origin`; UI-Typ `Ausgruendung`, Unterreiter „Ausgründen“ |
| KI-Firmen beteiligen sich, Wegkaufen, KI-Investoren | `ventures::{ai_month, ai_pledges, ai_takeover, ai_spin_offs, ai_chance, serves}`, `catalog::VentureAiModel` (`ki`), `rng::Stream::VentureBids`; Meldungen `meldung.startup.{ausgruendung, uebernahme}`; Investoren außerhalb des Spiels `Holder::Investors` |
| Pro-rata-Zusage der Mutter | `ventures::advance` (Tochterfirma zahlt ihren Anteil der Runde) |
| Forschungsbonus (beim Scheitern) | `ventures::research_bonus`, `VentureStakeModel::research_bonus` (`forschungsbonus`) |
| Empfehlung einer Beteiligung, erwarteter Ertrag je Dollar | `ventures::{recommendations, expected_return}`, `decision::{Topic::Venture, ChoiceKind::Invest}` (`startup`, `beteiligen`), `VentureStakeModel::{min_return, cash_share}` (`rendite_mindest`, `einsatz_kasse`); `VentureView::expected_return` |
| KI stellt ein, Kompetenz durch Manager | `staffing::{ai_staffing, next_position, fill, poach_target, ai_competence}`, `AiState::{staff, skill}`, `rng::Stream::Staffing`; `AiHiringModel` (`markt.ki`) |
| Schätzfehler der KI (Kosten, Marge) | `ai::estimate`, `AiBehavior::estimate_error` (`verhalten.schaetzfehler`), `rng::Stream::Estimate` (B2) |
| Zoll, Einfuhrzoll, Durchschnittszoll | `catalog::TariffModel` (`zoelle`; `default` = `standard`, `countries` = `laender`, `groups` = `warengruppen`), Modul `tariffs` (`TariffTable::{new, rate, for_product, average}`), `GameState::tariffs` (abgeleitet), Kostenart `CostType::Customs` (W3) |
| Handelszone, Beitritt, Austritt | `catalog::TariffZone` (`zonen`, `mitglieder`), `tariffs::zones_of`; Texte `zoll.zone.<id>` |
| Handelssperre | `catalog::Embargo` (`sperren`), `tariffs::embargoes_of`, Fehler `CommandError::Embargo` (`fehler.befehl.handelssperre`) |
| Zolldynamik (nach 2026) | `catalog::TariffDynamics` (`dynamik`, `stufen`), `GameSettings::tariff_dynamics`, `GameState::tariff_offsets`, `tariffs::new_year`, `rng::Stream::Tariffs`; Sicht `NewGameOptions::{tariffs, default_tariffs}`, Anfrage `NewGameRequest::tariffs`; CLI `--zoelle`; Texte `zoll.dynamik.<id>` |
| Zölle im Länderdetail, Kartenebene Zölle | `views::TariffView` (`CountryDetail::tariffs`), `MapCountry::tariff`; UI `Zollsaetze` (`Laenderdetail.tsx`), Ebene `zoll` (`Weltkarte.tsx`), Typ `Zoelle` |
| Liefervertrag, Monatsmenge, Vertragsstrafe, Kündigung | `contracts::{Contract, ContractStatus, Shortfall, Terms, Decline}`, `GameState::{contracts, next_contract}`, Befehle `ProposeContract`, `AnswerContract`, `CancelContract`; `contracts::{propose, answer, cancel, deliver, month_start, ai_answer, ai_proposals, best_partner, delivery_cost, ask_price, daily_output, daily_need, contracted}`; Daten `vertraege` (`ContractModel`, `ContractAiModel`), `rng::Stream::Contracts` (W4) |
| Vertragsangebot der KI, Abnehmer, Lieferant | `contracts::ai_proposals`; Meldungen `meldung.vertrag.{angebot, verfallen, beendet, fehlmenge_eigen, fehlmenge_partner}`; Fehler `CommandError::{NoContracts, ContractWithItself, UnknownContract, ContractDeclined}` |
| Lieferverträge (Ansicht) | Sichten `views::{ContractsView, ContractView, ContractSiteView, ContractPartnersView, ContractPartnerView}`, Sitzung `contracts`, `contract_partners`; Tauri/Web `vertraege`, `vertragspartner`; UI `VertraegeAnsicht` (`Vertraege.tsx`, Markt → „Lieferverträge“), Typen `Vertraege`, `Vertrag`, `Vertragspartnerliste` |
| Logistik, Weg der Ladungen (Frachtmarkt, staatlicher Transport, eigene Flotte) | `logistics::{Logistics, FreightMode::{Market, State, Fleet}, FleetHolding, Load, LogisticsMonth}`, `Company::logistics`; Befehle `BuyVehicles`, `SellVehicles`, `SetLogistics`; Daten `logistik` (`LogisticsModel`), Verkehrsmittel `nutzlast_t`/`kaufpreis_usd` (`Vehicle::fleet`, `FleetVehicle`) (W5) |
| Kapazität, Marktsatz, Betriebsanteil, Fracht für andere | `logistics::{capacity_per_vehicle, market_rate, running_share, running_share_first, price, for_sale}`, `logistics::{plan, book, month_end}`; `Routes::distance_km` |
| Verlorene Ladung, Frachtrisiko | `logistics::{risk, note_loss, loss_message}`, `Shipment::lost`, `rng::Stream::Freight`; Meldung `meldung.logistik.verlust`; Fehler `CommandError::{NoLogistics, VehicleNotForFleet, TooManyVehicles}` |
| KI-Flotte | `logistics::ai_purchases`, `ai::fleet` |
| Tochterfirma, Mutter, Konzern, Schwerpunkt (Produktion und Handel, Logistik) | `group::{SubsidiaryOf, SubsidiaryFocus::{Production, Logistics}, top, same_group, members, is_subsidiary}`, `Company::subsidiary_of`; Befehle `FoundSubsidiary`, `MoveCapital`, `TransferSite`, `SetSubsidiaryFocus`; Daten `tochterfirmen` (`SubsidiaryModel`) (W6) |
| Einlage, Ausschüttung, Standortübertragung zu Buchwerten | `group::{found, move_capital, transfer_site, set_focus}`; Fehler `CommandError::{NoSubsidiaries, CapitalTooLow, NotOwnSubsidiary, WithinGroup}` |
| Konzernbilanz, Konzern-GuV, Aufrechnung der Beteiligungen | `group::{Consolidated, consolidated}`; Pleite einer Tochter `group::settle_failures`, Meldung `meldung.tochter.pleite` |
| Frachtmarkt (Tonnenkilometer der Händler und Marktladungen), Vermietungsdeckel | `logistics::{FreightMarket, note_market_freight, rental_purchase}`, `GameState::freight_market` |
| Tochterfirmen (Ansicht) | Sichten `views::{GroupView, SubsidiaryView, GroupSiteView, GroupLineView}`, Sitzung `group`; Tauri/Web `konzern`; UI `TochterfirmenAnsicht` (`Tochterfirmen.tsx`, Organisation → „Tochterfirmen“), Typen `Konzern`, `Tochter`, `KonzernStandort`, `KonzernZeile` |
| Controlling, Deckungsbeitrag I/II, Fixkosten, variable Kosten, Vorperiode | Sicht `views::{controlling, ControllingView, ControllingNode, CostLineView}` (Blöcke `Block::{Revenue, Variable, Fixed, Other}`), Sitzung `controlling`; Tauri/Web `controlling`; UI `ControllingAnsicht` (`Controlling.tsx`, Finanzen → „Controlling“), Typen `Controlling`, `ControllingKnoten` (W7) |
| Börse, Börsenwert, Kurs, Index, Stimmung, Krise | `stock::{Listing, StockMarket, SHARES, target, earnings, month_start}`, `Company::listing`, `GameState::stock`, `rng::Stream::Stock`; Daten `boerse` (`StockModel`); Meldung `meldung.boerse.krise` (K1) |
| Börsengang, Kapitalerhöhung, Streubesitz, Ausschüttungsquote, Dividende | `stock::{go_public, issue_shares, issue_value, issue_proceeds, free_float, stake, set_dividend, dividend}`, `Company::dividend_payout`, `StockMarket::player_dividends`; Befehle `GoPublic`, `IssueShares`, `SetDividend`; Fehler `CommandError::{NoStockMarket, NotListed, AlreadyListed, EquityTooLow, ShareOutOfRange}`; Meldungen `meldung.boerse.{dividende_gezahlt, dividende_erhalten}` |
| Aktienkauf und -verkauf, Einstand, Abschreibung bei Pleite | `stock::{buy, sell, buy_price, sell_proceeds}`, `Company::stock_cost`; Befehle `BuyShares`, `SellShares`; Fehler `CommandError::{NotEnoughFreeFloat, NotEnoughStock}`; Meldung `meldung.boerse.abgeschrieben`; KI-Börsengang `stock::ai_ipo` |
| Anleihe, Kupon, Bonität, Zinsdeckung, Rückkauf | `bonds::{Bond, Standing, standing, grade_for, coupon, max_amount, issue, redeem, redeem_price, month_end, ai_prefers}`, `Company::bonds`, Konto `Account::Bonds`; Daten `anleihen` (`BondModel`, `BondGrade`); Befehle `IssueBond`, `RedeemBond`; Fehler `CommandError::{NoBonds, BondTerm, BondTooSmall, BondCompanyTooSmall, NoBondInvestors, UnknownBond}`; Meldung `meldung.anleihe.getilgt`; Sicht `views::{BondsView, BondView, BondQuoteView}` in `FinanceView::bonds`; UI `AnleihenTeil` (`Anleihen.tsx`, Finanzen → Abschluss), Typen `Anleihen`, `Anleihe` (K2) |
| Fairer Wert, Übernahmeangebot, Übernahmeprämie, feindliche Übernahme, Aktienrückkauf, KI-Anleger | `stock::{fair, takeover_price, take_over, buy_back, loses_majority, ai_trades, ai_takeover}`, `Listing::shares`; Befehle `TakeOver`, `BuyBackShares`; Fehler `CommandError::TakeoverNoMajority`; Meldungen `meldung.boerse.{uebernahme, uebernahme_verkauft}`, `meldung.spielende_uebernahme`; KI `ai::invest`; Sicht `OwnerView`, `OwnListingView::{buyback, owners}`, `ListedCompanyView::{fair_usd, takeover_usd}` (K3) |
| Investmentfirma, Bank (Startformen), Schwerpunkt Investment und Bank | `StartForm::{Investor, Bank}`, `SubsidiaryFocus::{Investment, Bank}` (K4) |
| Bank, Einlagen, Einlagenzins, Mindestreserve, Kreditnachlass, Kreditstandard, Ausleihungen, Kreditausfall | `bank::{BankSettings, deposit_rate, deposit_target, lending_room, lender_for, lend, receive, set, month_end, write_off_failures}`, `Company::bank`, `Loan::lender`, Konten `Account::{Deposits, LoansGiven}`; Daten `bank` (`BankModel`); Befehl `SetBank`; Fehler `CommandError::{NoBanks, NotABank, InvalidBankSettings}`; Meldung `meldung.bank.ausfall`; Sicht `views::{bank_view, BankView, BankRowView, BankLoanView}`, Sitzung `bank`; Tauri/Web `bank`; UI `BankAnsicht` (`Bank.tsx`, Finanzen → „Bank“), Typen `Bank`, `BankZeile`, `BankKredit` |
| Ereignisfolgen, Wirkung, Handelssperre (aus Ereignis), Zollaufschlag, Einberufung, Produktionskürzung, Abschottung, Zerstörung, Enteignung, Entschädigung, Staatsbetrieb, Börsenkrach, Folgen historischer Ereignisse | Modul `events` (`EventTable`, `active`, `active_in`, `month_on_or_after`, `closed_to`, `closed_to_newcomers`, `crashes`, `effect_messages`, `active_messages`, `simulate_day`, `EFFECT_TEXTS`), `GameState::events`, `TariffTable::{with_events, blocked}`, `Limit::Event`; Katalog `HistoricalEvent::effects`, `EventEffect`, `EffectKind::{Demand, Embargo, Tariff, Labor, Production, Closure, Destruction, Expropriation, StockCrash}`, `EventModel` (Daten `ereignisfolgen`); `Company::state_owned`, `ai::push_company`; Einstellung `GameSettings::event_effects` (`NewGameRequest::event_effects`, CLI `--ohne-folgen`); Fehler `CommandError::CountryClosed`; Meldungen `meldung.folge.*`, `meldung.ereignis.{zerstoerung, enteignung, staatsbetrieb}`; `Param::TextKeys`; Sicht `CountryDetail::events`; UI `folgenVon` (`Weltereignis.tsx`), Länderansicht „Folgen von Weltereignissen“ |
| Börse (Ansicht) | Sicht `views::{stock, StockMarketView, ListedCompanyView, OwnListingView, StockQuoteView, IssueQuoteView}`, Sitzung `stock`; Tauri/Web `boerse`; UI `BoerseAnsicht` (`Boerse.tsx`, Finanzen → „Börse“), Typen `Boerse`, `BoersenFirma`, `EigeneNotierung`, `AktienKurs`, `AktienAusgabe` |
| Logistik (Ansicht) | Sichten `views::{LogisticsView, FleetHoldingView, VehicleOfferView, LogisticsMonthView}`, Sitzung `logistics`; Tauri/Web `logistik`; UI `LogistikAnsicht` (`Logistik.tsx`, Markt → „Logistik“), Typen `Logistik`, `Flottenbestand`, `Fahrzeugangebot`, `Logistikmonat` |
