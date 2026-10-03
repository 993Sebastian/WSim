# Offene Punkte zum Lastenheft

Stand 03.10.2026. Entschiedene Punkte stehen auch in §18 des Lastenhefts.

**Status:** ✅ entschieden · 🟡 Vorschlag gilt vorläufig (nicht ausdrücklich bestätigt,
bei Bedarf widersprechen) · ❓ offen

## A Abgrenzung Stufe 1 gegenüber späteren Stufen

1. 🟡 **Transport in Stufe 1.** Ein abstrakter „Frachtdienst“ mit Kosten und Dauer nach
   Entfernung, Transportklasse und Epoche; keine Kapazitätsgrenzen, kein Risiko.
   Eigene Flotte, staatlicher und KI-Transport folgen in Stufe 2. Umgesetzt mit M8:
   Wege zwischen Hauptstädten, Seewege nach Luftlinie × Umwegfaktor (echte Seewege mit
   Kanälen ab Stufe 2), Waren bewegen sich innerhalb eines Landes ohne Kosten.
2. 🟡 **Zölle.** Stufe 1 ohne Zölle; die Warengruppen sind im Datenformat schon da.
3. 🟡 **Währung.** Stufe 1 rechnet und zeigt alles in USD (Kaufkraft 2026).
4. 🟡 **Strategie-Ansicht (§5.6)** kommt mit dem Manager-System in Stufe 2. In Stufe 1
   stellt der Spieler die Verkaufswege (§9.2) direkt je Produkt bzw. pauschal ein.
5. 🟡 **Insolvenz.** Einfache Insolvenz in Stufe 1 (Firma scheidet aus, Standorte
   werden stillgelegt oder günstig verkauft). Spielende für den Spieler nach §11.3
   vereinfacht: zahlungsunfähig und kein Kredit mehr möglich.
6. 🟡 **Forschung in Stufe 1** mit Technologiebaum, Forschungszentren,
   Länder-Forschungsstärke, Vorgriffskosten und Nachzügler-Rabatt; Patente und
   Lizenzen in Stufe 5.
7. 🟡 **Spieleinstellungen (§15) in Stufe 1:** Startjahr (1900–1930), Startland,
   Startkapital, Startform, Schwierigkeitsgrad, Anzahl/Kompetenz/Aggressivität der
   KI-Firmen, Preis-/Qualitätsempfindlichkeit, Konjunkturstärke,
   Forschungs-Kostenfaktor, Seed.

## B Fehlende Glieder in den 12 Ketten (§17.1)

8. ✅ **Güter außerhalb der acht Rohstoffe** (Glas, Zinn, Schwefel, Konserveninhalt,
   Leim …) werden vom **staatlichen Markt** bezogen. Umsetzung: Feld `staatsmarkt`
   am Produkt mit Preis und Verfügbarkeitsjahren; der Staatsmarkt bietet die Ware in
   jedem Land an, für Firmen und bei Konsumgütern auch für Endkunden.
9. ✅ **Pferdekutsche** ebenfalls über den Staatsmarkt; ihre Nachfrage sinkt mit der
   Verbreitung des Automobils. Die Petroleumlampe hat mit Kette 9 eine eigene Kette.
10. 🟡 **Koks.** Antwort „Staatsmarkt“ auf 8–10 so ausgelegt: Die Verkokung steckt
    weiter im Hochofen-Rezept (Kette 1 bleibt „Eisenerz + Kohle → Roheisen“); Koks
    ist kein eigenes Produkt. Bitte widersprechen, falls Koks als Staatsmarkt-Ware
    gemeint war.

## C Daten und Realismus

11. 🟡 **Heutige Grenzen.** Historische Werte werden über Flächen- und
    Bevölkerungsanteile umgerechnet und als Annäherung gekennzeichnet.
12. 🟡 **Länderliste.** 193 UN-Mitglieder plus Taiwan, Kosovo, Palästina, Westsahara;
    Kleinststaaten mit kleinen Werten.
13. ✅ **Länderwerte zeigen den realen Verlauf** einschließlich Kriegs- und
    Krisendellen. Folge für Stufe 4: Ereignisse dürfen diese Einbrüche nicht noch
    einmal erzeugen; sie wirken über das hinaus, was die Zeitreihen schon enthalten
    (Zerstörung von Standorten, Embargos, Rüstungsnachfrage, Arbeitskräftemangel).
14. 🟡 **Kaufkraft und Löhne.** Jedes Land bekommt einen Preisniveau-Faktor
    (Marktkurs/Kaufkraftparität) als Jahreswert.
15. 🟡 **Arbeitskräfte:** Vorschlag siehe unten, im Datenformat umgesetzt; Freigabe ausstehend.
16. 🟡 **Lagerstätten.** Je Land und Rohstoff die bedeutenden Lagerstätten einzeln,
    dazu ein zusammengefasstes „Restvorkommen“; mit Jahr der Entdeckung.
17. 🟡 **Besitz zum Startjahr.** Bereits betriebene Minen und Werke gehören den
    KI-Firmen der Startbesetzung; nicht erschlossene Lagerstätten sind frei.
18. 🟡 **Stromnetz 1900.** Jahreswert „Netzverfügbarkeit“ je Land; ohne Netz nur
    Eigenerzeugung (Kette 7) oder Kohle/Dampf.

## D Umfang und Technik

19. 🟡 **Anzahl KI-Firmen.** Ausgelegt auf 10 000, Stufe 1 getestet mit 1 000,
    Standard 300.
20. 🟡 **Reale Firmen.** Stufe 1 etwa 30 reale Firmen der Kernbranchen plus
    generierte; vollständige Liste in Stufe 5.
21. ✅ **Sprache.** Code-Bezeichner englisch; Datendateien und alle Texte deutsch.
22. 🟡 **Formelabnahme.** Formeln je Meilenstein in `docs/FORMELN.md`, Freigabe mit
    dem Meilenstein.

## Vorschlag zu Punkt 15: Arbeitskräfte

Vier **Qualifikationen**, bei Fachkräften und Akademikern zusätzlich eine von neun
**Fachrichtungen**. Je Land gibt es damit 2 + 2 × 9 = **20 Arbeitskräftegruppen**
mit eigenem Pool und eigenem Lohnniveau.

| Qualifikation | Typisch um 1900 | Fachrichtung |
| --- | --- | --- |
| Ungelernte | Hilfsarbeiter, Handlanger, Landarbeiter | – |
| Angelernte | Maschinenbediener; ab 1913 Fließband | – |
| Fachkräfte | Hauer, Schmelzer, Schlosser, Weber, Meister | ja |
| Akademiker | Ingenieure, Chemiker, Kaufleute mit Studium, Forscher | ja |

| Fachrichtung | Deckt ab (Branchen nach §6.2) |
| --- | --- |
| Bergbau | Bergbau und Rohstoffe, Ölförderung |
| Metall | Metallurgie, Gießerei, Metallbearbeitung |
| Maschinenbau | Maschinenbau, Fahrzeuge, Schiffbau, Luft- und Raumfahrt, Rüstung |
| Elektrotechnik | Elektrotechnik und Elektronik, Energie |
| Chemie | Chemie, Raffinerie, Gummi, Glas, Pharma |
| Textil | Textil, Bekleidung, Leder |
| Holz und Bau | Holz und Papier, Möbel, Bau |
| Lebensmittel | Lebensmittel |
| Kaufmännisch | Einkauf, Vertrieb, Verwaltung, Logistik |

Warum so:

- **Angelernte als eigene Stufe** bilden die Fließbandfertigung ab: Ford ersetzte
  Facharbeiter durch Angelernte. Automatisierung und Fließband verschieben den Bedarf
  eines Rezepts von Fachkräften zu Angelernten – so wird die Länderprägung
  „Handmontage oder Automatisierung“ (§3.2) spielbar.
- **Fachrichtungen = Forschungsfachgebiete:** Die Forschungsstärke eines Landes
  (§3.2) hängt an denselben Akademikern, die auch in der Produktion arbeiten.
- **Neun Fachrichtungen** reichen, um Standortvorteile zu zeigen (Ruhrgebiet:
  Bergbau und Metall; Lancashire: Textil), ohne die Länderdaten unüberschaubar zu
  machen.

Regeln für M5 (Formeln folgen in `docs/FORMELN.md`):

- Eine höhere Qualifikation kann eine niedrigere vertreten (volle Leistung, höherer Lohn).
- Eine Fachkraft einer anderen Fachrichtung arbeitet mit verminderter Leistung
  (Vorschlag 50 %).

Qualifikationen und Fachrichtungen stehen in `data/arbeitskraefte.yaml` und lassen
sich ohne Programmierung ändern.
