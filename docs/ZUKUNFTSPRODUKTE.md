<!-- Vorgabe des Auftraggebers vom 14.10.2026, übernommen aus dem Dokument „WSim – Produkte ab 2026“ (Claude Docs). Meilenstein ZP1. -->
# WSim – Produkte ab 2026

2026-10-14 · Sebastian

## Einordnung

Vorschlag: 92 neue Produkte und Vorprodukte zwischen 2027 und 2085, verteilt auf alle 17 Branchen und angeschlossen an Technologien, die es im Spiel schon gibt. Grundlage ist der Datenstand auf GitHub (Branch claude/architektur-vorschlag, 12.10.2026, rund 300 Endprodukte bis 2026). Was Claude Code seitdem nur lokal ergänzt hat, konnte ich nicht sehen; Doppelungen bitte markieren.

**Wie die Jahre gemeint sind:** "ab" ist das früheste plausible Erfindungsjahr, also der Wert für `erfindungsjahr`. Drei Stufen:

- **Nah (2027–2035):** angekündigte oder in Pilotfertigung befindliche Produkte, recht sicher.
- **Mittel (2036–2060):** technisch absehbar, Zeitpunkt unsicher.
- **Fern (2061–2100):** spekulativ, sorgt für Ziele im Spätspiel.

**Was das am Lastenheft ändert:** Der Technologiestopp 2026 (Lastenheft §3 und §6) fällt. Ebenso die Start-up-Regel, nach der es ab 2026 keine Start-ups für neue Technologien mehr gibt (BETEILIGUNGEN §5.2).

**Getroffene Annahmen (bis auf Widerruf):**

1. **Jahre je Spiel verschoben:** Das Erfindungsjahr wird bei Spielstart zufällig verschoben, Nah ±2, Mittel ±5, Fern ±10 Jahre. So weiß auch ein erfahrener Spieler nicht genau, wann was kommt. Die Verschiebung gilt für Spieler und KI gleich.
2. **Erfindung wie bei historischen Produkten:** Zum verschobenen Jahr gilt die Technologie als in der Welt erfunden; danach wird Nachforschen günstiger (Nachzügler-Rabatt). Herstellen darf trotzdem nur, wer sie selbst erforscht, lizenziert oder per Start-up erworben hat (Forschungspflicht). Früher forschen geht wie bisher mit Vorgriff-Kosten.
3. **Start-ups für neue Technologien auch nach 2026:** Die Sperre in BETEILIGUNGEN §5.2 entfällt; Zukunftsprodukte sind gute Start-up-Ziele.
4. **Alle drei Stufen werden aufgenommen.** Fern-Produkte bekommen `annaeherung: true` und werden nach den ersten Testpartien nachjustiert.
5. **Energie wird ausgebaut** mit Wasserstoff, Methanol und E-Fuels; Grundprodukt der Branche bleibt das bisherige.
6. **Werte wie Preise, Anlagen und Nachfrage** legt Claude Code beim Umsetzen fest, als Annäherung nach dem Muster der bestehenden Ketten.

Spalten der Tabellen: Produkt (vorgeschlagene ID), ab Jahr, Art (End = Endprodukt, Vor = Vorprodukt/Komponente), Voraussetzung (bestehende Technologie-IDs), ersetzt (bestehende Produkt-IDs), Kurzbeschreibung.

## Energie, Bergbau, Metallurgie

Wasserstoff ist der Schlüssel dieses Blocks: Ohne ihn gibt es keinen grünen Stahl, keine E-Fuels und kein grünes Ammoniak.

| Produkt | ab | Art | Branche | Voraussetzung | ersetzt | Kurzbeschreibung |
|---|---|---|---|---|---|---|
| gruener_wasserstoff | 2027 | Vor | energie | elektrolyseur (neu) | – | Wasserstoff per Elektrolyse aus Strom; Grundstoff für Stahl, Ammoniak, E-Fuels |
| gruenes_methanol | 2028 | Vor | energie | gruener_wasserstoff | – | Schiffskraftstoff und Chemiegrundstoff |
| e_kerosin | 2030 | End | energie | gruener_wasserstoff, erdoeldestillation | kerosin (anteilig) | Synthetisches Flugbenzin; Beimischquoten erzeugen Staatsnachfrage |
| e_diesel | 2032 | End | energie | gruener_wasserstoff, erdoeldestillation | dieselkraftstoff (anteilig) | Für Schiffe, Landmaschinen und den Fahrzeugbestand |
| lithium_sole | 2028 | Vor | bergbau | – | lithium (anteilig) | Lithium per Direktextraktion aus Sole und Thermalwasser; neue Lagerstätten |
| manganknollen | 2033 | Vor | bergbau | bergbaumaschine (Produkt) | – | Tiefseebergbau für Nickel, Kobalt, Mangan, Kupfer; Umweltauflagen als Ereignis |
| asteroidenmetall | 2075 | Vor | bergbau | asteroidenbergbau (neu) | – | Platinmetalle und Nickel aus dem All; drückt die Weltmarktpreise |
| helium3 | 2085 | Vor | bergbau | mondbasis (neu) | – | Brennstoff für Fusionsreaktoren der zweiten Generation |
| gruener_stahl | 2027 | Vor | metallurgie | gruener_wasserstoff, elektrostahl | stahl (anteilig) | Wasserstoff-Direktreduktion plus Elektroofen; teurer, rechnet sich mit CO₂-Kosten |
| batterierecyclat | 2027 | Vor | metallurgie | lithium_ionen_akku | lithium, kobalt (anteilig) | Rohstoffe aus Altbatterien; Menge wächst mit dem Elektroauto-Bestand |
| magnet_ohne_seltene_erden | 2031 | Vor | metallurgie | elektrostahl | – | Eisennitrid-Magnete für Elektromotoren und Windkraft |
| hochentropie_legierung | 2042 | Vor | metallurgie | – | titan (anteilig) | Hitzefeste Werkstoffe für Triebwerke und Fusionsreaktoren |

## Chemie, Pharma

Pharma bringt teure Produkte mit kleinen Stückzahlen; die Verjüngungstherapie könnte zusätzlich an das Personensystem koppeln und die Lebenserwartung von Spielerfigur und Managern verlängern.

| Produkt | ab | Art | Branche | Voraussetzung | ersetzt | Kurzbeschreibung |
|---|---|---|---|---|---|---|
| gruenes_ammoniak | 2028 | Vor | chemie | gruener_wasserstoff, haber_bosch | ammoniak (anteilig) | Für CO₂-armen Dünger und als Schiffskraftstoff |
| biokunststoff_pha | 2029 | Vor | chemie | gentechnische_wirkstoffe | polyethylen (anteilig) | Biologisch abbaubarer Kunststoff aus Reststoffen, industrieller Maßstab |
| co2_kunststoff | 2031 | Vor | chemie | co2_abscheidung (neu), polypropylen | polypropylen (anteilig) | Kunststoff aus abgeschiedenem CO₂ und Wasserstoff |
| mikrobieller_duenger | 2030 | End | chemie | gentechnische_wirkstoffe | stickstoffduenger (anteilig) | Bakterien binden Stickstoff an der Wurzel |
| graphen | 2033 | Vor | chemie | kohlefaser | – | Graphen in Industriemengen für Akkus, Beton und Leiter |
| selbstheilender_lack | 2036 | End | chemie | farbe_lack (Produkt) | farbe_lack | Lack, der Kratzer selbst schließt; Auto und Bau |
| orales_abnehmmittel | 2027 | End | pharma | gentechnische_wirkstoffe | – | Stoffwechselmedikament als Tablette; Massenmarkt in reichen Ländern |
| gentherapie | 2028 | End | pharma | gentechnische_wirkstoffe | – | Einmalbehandlung erblicher Krankheiten; sehr teuer, kleine Mengen |
| krebsimpfstoff | 2030 | End | pharma | impfstoffproduktion, gentechnische_wirkstoffe | – | Personalisierter mRNA-Impfstoff gegen Tumore |
| alzheimer_medikament | 2034 | End | pharma | gentechnische_wirkstoffe | – | Bremst den Verlauf; Nachfrage wächst mit alternder Bevölkerung |
| breitband_virostatikum | 2040 | End | pharma | gentechnische_wirkstoffe | – | Wirkt gegen ganze Virusfamilien; Staatsnachfrage bei Pandemie-Ereignissen |
| biogedrucktes_organ | 2048 | End | pharma | bioprinting (neu) | – | Organe aus eigenen Zellen; ersetzt Spenderorgane |
| verjuengungstherapie | 2065 | End | pharma | gentherapie (neu) | – | Verlangsamt das Altern; nur für sehr hohe Einkommen |

## Elektro, Maschinenbau

Roboter sind hier der große Hebel: Humanoide und Haushaltsroboter senken den Arbeitsbedarf in Werken und schaffen einen neuen Konsummarkt.

| Produkt | ab | Art | Branche | Voraussetzung | ersetzt | Kurzbeschreibung |
|---|---|---|---|---|---|---|
| natrium_ionen_akku | 2027 | Vor | elektro | lithium_ionen_akku | lithium_ionen_akku (anteilig) | Ohne Lithium und Kobalt; billiger, schwerer; für Speicher und Kleinwagen |
| tandem_solarmodul | 2028 | End | elektro | perc_solarzelle | solarmodul | Perowskit-Silizium-Tandem, rund 30 % Wirkungsgrad |
| ar_brille | 2028 | End | elektro | smartphone | smartphone (anteilig) | Datenbrille mit Einblendungen; Alltagsgerät ab den 2030ern |
| ki_assistent | 2028 | End | elektro | smartphone | – | Kleines Gerät ohne Bildschirm, per Sprache gesteuert |
| festkoerperbatterie | 2029 | Vor | elektro | traktionsbatterie | traktionsbatterie | Mehr Reichweite, schneller ladbar, nicht brennbar |
| mikroled_fernseher | 2029 | End | elektro | flachbildfernseher | flachbildfernseher | Heller und sparsamer; erst teuer, dann Standard |
| haushaltsroboter | 2032 | End | elektro | industrieroboter | staubsauger (anteilig) | Räumt auf, wäscht, kocht einfache Gerichte |
| photonischer_chip | 2033 | Vor | elektro | mikroprozessor | mikrochip (anteilig) | Rechnet mit Licht; spart Strom in Rechenzentren |
| quantencomputer | 2033 | End | elektro | integrierte_schaltung | – | Investitionsgut; könnte Forschung in Chemie und Pharma beschleunigen |
| gehirn_schnittstelle | 2042 | End | elektro | ar_brille (neu) | ar_brille (anteilig) | Gerätesteuerung per Gedanken; erst Medizin, dann Konsum |
| holo_display | 2045 | End | elektro | mikroled_fernseher (neu) | mikroled_fernseher | Räumliches Bild ohne Brille |
| elektrolyseur | 2027 | End | maschinenbau | elektrizitaetswerk | – | Anlage für grünen Wasserstoff; Investitionsgut |
| brennstoffzelle | 2028 | Vor | maschinenbau | elektrolyseur (neu) | – | Stack für Lkw, Schiffe und Notstrom (vor 2026 nur Kleinserie) |
| metall_3d_drucker | 2028 | End | maschinenbau | cnc_steuerung | werkzeugmaschine (anteilig) | Serienfertigung komplexer Metallteile |
| agrarroboter | 2029 | End | maschinenbau | ackerschlepper, industrieroboter | traktor (anteilig) | Autonome Feldarbeit, Unkrautjäten ohne Gift |
| exoskelett | 2029 | End | maschinenbau | industrieroboter | – | Kraftunterstützung in Fertigung, Logistik und Pflege |
| humanoider_roboter | 2030 | End | maschinenbau | industrieroboter | – | Allzweck-Arbeitskraft in Werk und Lager |
| co2_abscheider | 2030 | End | maschinenbau | luftzerlegung | – | Filtert CO₂ aus der Luft; lohnt erst bei hohen CO₂-Preisen |
| modularer_reaktor | 2032 | End | maschinenbau | turbogenerator | – | Kleines Kernkraftwerk aus der Fabrik, rund 300 MW; für Versorger und Staat |
| fusionskraftwerk | 2055 | End | maschinenbau | modularer_reaktor (neu), hochentropie_legierung (neu) | – | Strom aus Kernfusion; senkt langfristig die Strompreise |

## Fahrzeuge, Luftfahrt, Raumfahrt, Schiffbau

Autonomes Fahren verändert die Nachfrage am stärksten: Robotaxis senken den privaten Autobesitz, fahrerlose Lkw die Transportkosten im Spiel.

| Produkt | ab | Art | Branche | Voraussetzung | ersetzt | Kurzbeschreibung |
|---|---|---|---|---|---|---|
| robotaxi | 2028 | End | fahrzeuge | elektroauto | automobil (anteilig) | Fahrerloses Taxi für Flottenbetreiber; weniger Autobesitz in Städten |
| wasserstoff_lkw | 2028 | End | fahrzeuge | lastkraftwagen, brennstoffzelle (neu) | lastwagen (anteilig) | Schwerer Fernverkehr ohne lange Ladepausen |
| lieferroboter | 2028 | End | fahrzeuge | industrieroboter | lieferwagen (anteilig) | Autonome Zustellung auf der letzten Meile |
| autonomer_lkw | 2031 | End | fahrzeuge | lastkraftwagen | lastwagen (anteilig) | Fernstrecke ohne Fahrer; senkt Transportkosten |
| autonomes_elektroauto | 2033 | End | fahrzeuge | elektroauto, festkoerperbatterie (neu) | elektroauto | Privatwagen, der vollständig selbst fährt |
| vakuumbahn | 2050 | End | fahrzeuge | hochgeschwindigkeitszug | hochgeschwindigkeitszug (anteilig) | Kapseln in Unterdruckröhren, über 800 km/h |
| evtol_lufttaxi | 2027 | End | luftfahrt | multikopter, traktionsbatterie | – | Elektrisches Lufttaxi für Städte und Flughafenzubringer |
| frachtdrohne | 2028 | End | luftfahrt | multikopter | – | Pakete und Ersatzteile, auch in entlegene Gebiete |
| elektroflugzeug | 2030 | End | luftfahrt | traktionsbatterie, turbopropflugzeug | regionalflugzeug (anteilig) | Rund 19 Sitze, Kurzstrecke bis etwa 400 km |
| ueberschall_jet | 2030 | End | luftfahrt | duesenverkehrsflugzeug | geschaeftsreiseflugzeug (anteilig) | Überschall-Passagierflugzeug für Langstrecke über Wasser |
| wasserstoff_flugzeug | 2037 | End | luftfahrt | brennstoffzelle (neu), mantelstromtriebwerk | verkehrsflugzeug (anteilig) | Mittelstrecke mit Wasserstoff; braucht passende Flughäfen |
| hyperschall_flugzeug | 2065 | End | luftfahrt | ueberschall_jet (neu) | grossraumflugzeug (anteilig) | Europa–Australien in rund vier Stunden |
| superschwerlastrakete | 2027 | End | raumfahrt | wiederverwendbare_rakete | traegerrakete | Voll wiederverwendbar; senkt Startkosten je kg stark |
| mondlander | 2028 | End | raumfahrt | raumsonde | – | Fracht und Menschen zum Mond; Staatsnachfrage |
| raumstationsmodul | 2029 | End | raumfahrt | traegerrakete | – | Kommerzielle Raumstation; Forschung und Fertigung im All |
| orbitalreise | 2032 | End | raumfahrt | superschwerlastrakete (neu) | – | Mehrtägige Reise ins All; Luxusgut |
| mondbasis | 2045 | End | raumfahrt | mondlander (neu) | – | Dauerhafte Station; Grundlage für Helium-3 |
| weltraumsolarkraftwerk | 2055 | End | raumfahrt | superschwerlastrakete (neu), tandem_solarmodul (neu) | – | Sendet Solarstrom per Mikrowelle zur Erde |
| asteroidenbergbau | 2070 | End | raumfahrt | superschwerlastrakete (neu) | – | Sonde, die Metalle abbaut und zurückbringt |
| marshabitat | 2078 | End | raumfahrt | mondbasis (neu) | – | Erste Siedlung auf dem Mars; Prestigeprojekt der Staaten |
| ammoniak_containerschiff | 2028 | End | schiffbau | containerschiff, gruenes_ammoniak (neu) | containerschiff (anteilig) | Emissionsfreier Seetransport |
| autonomes_frachtschiff | 2032 | End | schiffbau | motorschiff | frachtschiff (anteilig) | Fährt ohne Besatzung; geringere Betriebskosten |

## Lebensmittel, Textil, Konsumgüter, Bau, Holz/Papier, Rüstung

Bei Lebensmitteln und Bau verdrängen neue Produkte alte nur teilweise; das passt zur bestehenden Ersatzlogik über `ersetzt`.

| Produkt | ab | Art | Branche | Voraussetzung | ersetzt | Kurzbeschreibung |
|---|---|---|---|---|---|---|
| insektenprotein | 2027 | End | lebensmittel | spruehtrocknung | tierfutter (anteilig) | Mehl und Riegel aus Insekten; zuerst als Tierfutter |
| mikrobielles_protein | 2028 | Vor | lebensmittel | gruener_wasserstoff (neu) | – | Proteinpulver aus Bakterien, Luft und Strom |
| indoor_gemuese | 2028 | Vor | lebensmittel | led_lampe | obst_gemuese (anteilig) | Vertical Farming: ganzjährig, stadtnah, ohne Pestizide |
| tierfreie_milch | 2029 | End | lebensmittel | gentechnische_wirkstoffe | trinkmilch, kaese (anteilig) | Milcheiweiß aus Präzisionsfermentation |
| kultiviertes_fleisch | 2031 | End | lebensmittel | gentechnische_wirkstoffe | wurst, gefluegel (anteilig) | Fleisch aus Zellkultur; Zulassung je Land als Ereignis |
| personalisierte_nahrung | 2045 | End | lebensmittel | metall_3d_drucker (neu) | fertiggericht (anteilig) | Gedruckte Mahlzeiten nach persönlichem Bedarf |
| myzelleder | 2028 | Vor | textil | – | leder (anteilig) | Lederersatz aus Pilzgeflecht |
| faserrecyclat | 2028 | Vor | textil | polyesterfaser | polyesterfaser (anteilig) | Neue Faser aus Alttextilien; Recyclingquoten als Regulierung |
| smarte_kleidung | 2031 | End | textil | funktionsfaser | funktionsjacke (anteilig) | Misst Puls und Temperatur, heizt oder kühlt |
| spinnenseide | 2036 | Vor | textil | gentechnische_wirkstoffe | – | Künstlich erzeugte Spinnenseide; extrem reißfest |
| ki_spielzeug | 2029 | End | konsumgueter | kunststoffspielzeug | kunststoffspielzeug (anteilig) | Spielzeug, das per KI spricht und lernt |
| personalisierte_kosmetik | 2030 | End | konsumgueter | emulsionscreme | hautcreme (anteilig) | Pflege nach Hautanalyse, individuell gemischt |
| co2_armer_zement | 2028 | Vor | bau | – | zement (anteilig) | Weniger Klinker, CO₂ wird abgeschieden; rechnet sich mit CO₂-Preis |
| myzel_verpackung | 2028 | End | bau | – | verpackungsschaum (anteilig) | Kompostierbarer Formschaum aus Pilzgeflecht |
| gedrucktes_haus | 2029 | End | bau | transportbeton | – | Rohbau aus dem 3D-Betondrucker in wenigen Tagen |
| solarfassade | 2031 | End | bau | tandem_solarmodul (neu) | fenster (anteilig) | Fassade und Fenster, die Strom erzeugen |
| selbstheilender_beton | 2036 | End | bau | transportbeton | transportbeton (anteilig) | Bakterien schließen Risse; längere Lebensdauer |
| holzbaumodul | 2029 | End | holz_papier | spanplatte | betonfertigteile (anteilig) | Brettsperrholz-Module für Holzhochhäuser |
| holzschaum_daemmung | 2032 | End | holz_papier | mineralwolle | daemmstoff (anteilig) | Dämmung aus Holzfasern, ohne Kunststoff |
| transparentes_holz | 2042 | End | holz_papier | spanplatte | fensterglas (anteilig) | Lichtdurchlässiges Holz; dämmt besser als Glas |
| laserwaffe | 2027 | End | ruestung | radar | – | Flugabwehr gegen Drohnen; sehr günstig je Schuss |
| drohnenschwarm | 2029 | End | ruestung | militaerdrohne | – | Viele kleine, gemeinsam gesteuerte Drohnen |
| begleitdrohne | 2029 | End | ruestung | militaerdrohne, duesenjaeger | – | Unbemannter Begleiter für Kampfjets |
| kampfjet_6_generation | 2036 | End | ruestung | duesenjaeger | kampfjet | Nächste Kampfjet-Generation mit Drohnensteuerung |
| railgun | 2045 | End | ruestung | – | geschuetz (anteilig) | Elektromagnetisches Geschütz für Schiffe |
