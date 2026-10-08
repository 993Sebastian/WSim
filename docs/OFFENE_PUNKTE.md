# Offene Punkte zum Lastenheft

Stand 06.10.2026. Entschiedene Punkte stehen auch in §18 des Lastenhefts.

**Status:** ✅ entschieden · 🟡 Vorschlag gilt vorläufig (nicht ausdrücklich bestätigt,
bei Bedarf widersprechen) · ❓ offen

## A Abgrenzung Stufe 1 gegenüber späteren Stufen

1. 🟡 **Transport in Stufe 1.** Ein abstrakter „Frachtdienst“ mit Kosten und Dauer nach
   Entfernung, Transportklasse und Epoche; keine Kapazitätsgrenzen, kein Risiko.
   Eigene Flotte, staatlicher und KI-Transport folgen in Stufe 2. Umgesetzt mit M8:
   Wege zwischen Hauptstädten, Seewege nach Luftlinie × Umwegfaktor (echte Seewege mit
   Kanälen ab Stufe 2), Waren bewegen sich innerhalb eines Landes ohne Kosten.
2. 🟡 **Zölle.** Stufe 1 ohne Zölle; die Warengruppen sind im Datenformat schon da.
   Umgesetzt mit W3 (`parameter/zoelle.yaml`, FORMELN W3): Durchschnittszoll je
   Einfuhrland als Zeitreihe (15 große Länder eigen, sonst ein Standardverlauf nach
   Bairoch/Clemens–Williamson), Faktor je Warengruppe, Handelszonen (EWG/EU, EFTA, NAFTA,
   Mercosur, Empire-Präferenz, RGW) und drei Handelssperren. Vorschläge: (a) Zoll auf den
   Einstandspreis (Preis + Fracht) der Händler und auf den Lagerwert eigener Lieferungen;
   ein Zoll auf den Rechnungswert ohne Fracht (FOB, wie in den USA) wäre etwas niedriger.
   (b) Die Zolleinnahmen verlassen das Spiel (keine Staatshaushalte in Stufe 2). (c) Nach
   2026 ändern sich die Zölle zufällig (keine, normal, stark schwankend; Wahl beim neuen
   Spiel); Handelskriege und Sanktionen als Ereignisse folgen mit Stufe 4.
3. 🟡 **Währung.** Stufe 1 rechnet in USD (Kaufkraft 2026). Die Anzeige in
   Landeswährungen ist mit M21 vorgezogen (§18.2): umschaltbar zwischen Kaufkraft 2026
   und Preisen der Zeit. Vorschlag: „Kaufkraft 2026“ zeigt die Landeswährung des Jahres
   2026 (Deutschland: Euro), weil es eine Mark „mit der Kaufkraft von 2026“ nicht gibt;
   die historischen Währungen mit ihren Umstellungen (Mark, Reichsmark, D-Mark) zeigt
   „Preise der Zeit“. Wechselkurswirkungen auf Handel, Löhne und Gewinne folgen mit
   Stufe 4.
4. 🟡 **Strategie-Ansicht (§5.6)** kommt mit dem Manager-System in Stufe 2. In Stufe 1
   stellt der Spieler die Verkaufswege (§9.2) direkt je Produkt bzw. pauschal ein.
5. 🟡 **Insolvenz.** Einfache Insolvenz in Stufe 1 (Firma scheidet aus, Standorte
   werden stillgelegt oder günstig verkauft). Spielende für den Spieler nach §11.3
   vereinfacht: zahlungsunfähig und kein Kredit mehr möglich.
6. 🟡 **Forschung in Stufe 1** mit Technologiebaum, Forschungszentren,
   Länder-Forschungsstärke, Vorgriffskosten und Nachzügler-Rabatt; Patente und
   Lizenzen in Stufe 5. Umgesetzt mit M9; zusätzlich wird jede Technologie 25 Jahre
   nach ihrer historischen Erfindung Gemeingut (Wert in den Daten).
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

19. ✅ **Anzahl KI-Firmen.** Standard **100 KI-Firmen** (Entscheidung des Auftraggebers,
    04.10.2026: „eine Welt mit 100 Live-Gegnern ist ausreichend“), einstellbar; getestet
    mit 1 000. Die **Marktgröße passt sich der Firmenzahl an**: Ein Marktmaßstab
    verkleinert alle Mengen (Endkunden- und Staatsnachfrage, Arbeitskräftepools,
    Förderung der Lagerstätten); Preise, Stundenlöhne, Einkommen je Kopf und
    Anlagengrößen bleiben real. Mehr Firmen → größere Märkte. Länderwerte werden weiter
    real angezeigt.
20. 🟡 **Reale Firmen.** Stufe 1 etwa 30 reale Firmen der Kernbranchen plus
    generierte; vollständige Liste in Stufe 5.
21. ✅ **Sprache.** Code-Bezeichner englisch; Datendateien und alle Texte deutsch.
22. 🟡 **Formelabnahme.** Formeln je Meilenstein in `docs/FORMELN.md`, Freigabe mit
    dem Meilenstein.
23. 🟡 **Handelsfirmen als KI.** In Stufe 1 bleiben die Händler ein abstrakter,
    wettbewerblicher Händlermarkt (M8); KI-Firmen produzieren. Vorschlag: KI-Handels-
    firmen mit eigener Bilanz zusammen mit Flotten in Stufe 2.
24. 🟡 **Einkäufe ohne Kassenprüfung.** Einkaufsaufträge (Spieler und KI) kaufen auch
    bei leerer Kasse; das Konto geht dann ins Minus bis zur Überziehungsgrenze, danach
    droht die Insolvenz. Vorschlag: so lassen (einfach, die Insolvenzprüfung fängt es).
25. 🟡 **Ersatz durch höhere Qualifikation.** Die Regel „höhere Qualifikation vertritt
    niedrigere“ (Vorschlag zu Punkt 15) ist noch nicht umgesetzt; fehlende Arbeitskräfte
    bremsen die Produktion. Vorschlag: mit dem Balancing in M15 nachziehen.
26. 🟡 **Werbemittel.** Vorschlag Stufe 1: Es wirkt automatisch das beste Werbemittel der
    Epoche (Zeitung, ab 1923 Radio); die Wahl einzelner Werbemittel folgt in Stufe 2.
27. 🟡 **Marke je Warengruppe.** Vorschlag: Bekanntheit je Firma, Land und Warengruppe
    (nicht je Produkt), wie im Lastenheft §9.4; Händlerware und Staatsmarkt mit festen
    Werten.
28. 🟡 **Tutorial-Stand beim Speichern.** Vorschlag: Das Tutorial gehört zur laufenden
    Sitzung; nach dem Laden eines Spielstands ist es aus und lässt sich über die
    Tastenhilfe neu starten.
29. 🟡 **Präsenz und Preisempfindlichkeit der Endkunden.** Umgesetzt (M16): Jedes Angebot
    zählt in der Anbieterwahl mit seiner Präsenz (Erzeugung je Tag plus Lager / 30), und
    das Preisgewicht liegt bei 7 (ärmstes Fünftel) bis 3 (reichstes) statt 2,0–0,6.
    Ohne beides verkaufte ein Neuling mit einer Maschine alles zu fast jedem Preis.
    Zu prüfen beim Spielen: ob Neulinge mit Werbung und Preis schnell genug Anteile
    gewinnen (Stellschrauben: `preisgewicht`, `markengewicht`, `kosten_je_einwohner_usd`).
30. 🟡 **Preise der Waren je Land (Nacharbeit zu M16).** Bisher galt der Richtpreis
    überall mal dem vollen Preisniveau des Landes. Umgesetzt: Waren folgen ihm nur mit
    einem Anteil je Produktart (`preisniveau_anteil`: Rohstoffe 0,2, Halbzeuge und
    Komponenten 0,1, Endprodukte 0,4, Strom 1). Grund: Eingeführte Vorprodukte kosten
    überall Weltpreise; mit dem vollen Faktor lagen Preise und Untergrenzen in ärmeren
    Ländern beim 1,5- bis 2-Fachen des Richtpreises, und Werke dort konnten nie zum
    Richtpreis verkaufen. Folge: Verbraucher ärmerer Länder kaufen weniger Industriewaren
    als bisher (realistisch, aber eine Änderung der Marktgrößen). Alternative: alle
    Anteile auf 1 setzen (bisheriges Verhalten).
31. 🟡 **Gemeinkosten (Nacharbeit zu M16).** Die Rezepte enthalten nur direkte Kosten.
    Umgesetzt: Verwaltung, Vertrieb und Logistik als Zuschlag auf die Umwandlungskosten
    (Arbeit, Strom, Anlage) je Produktart (Rohstoffe 25 %, Halbzeuge und Komponenten 50 %,
    Endprodukte 100 %), gebucht als Kostenart „Verwaltung und Vertrieb“. Ohne sie drückte
    der Wettbewerb die Preise auf 20–50 % des Richtpreises. Später (Stufe 2, Manager)
    könnten Gemeinkosten aus eigener Verwaltung und eigenem Vertrieb entstehen.
32. 🟡 **Arbeitsproduktivität nach Wohlstand (Nacharbeit zu M16).** Die Arbeitsstunden
    der Rezepte gelten bei 10.000 USD BIP je Kopf; Länder brauchen dazu im Verhältnis
    mehr oder weniger Stunden (Elastizität 1, `laendermodell.produktivitaet`). So kostet
    Arbeit je Stück in Kaufkraft überall gleich viel, und Niedriglohnländer haben keinen
    übertriebenen Kostenvorteil.
33. 🟡 **Plausibilitätsprüfung der Richtpreise.** `validate` warnt, wenn das beste Rezept
    eines Produkts zu Richtpreisen weniger als 5 % oder mehr als 45 % Marge bringt
    (Förderung mit ihrer Pacht, siehe 39). Die Prüfung fand sieben Unstimmigkeiten (Nägel,
    Handwerkzeug, Möbel, Glühlampe, Nähmaschine, Schnittholz, Benzin), korrigiert über
    Richtpreise, Ausbeuten und Arbeitsstunden (`docs/FORMELN.md`, Plausibilität). Bitte
    widersprechen, falls Richtpreise unverändert bleiben sollen.
34. 🟡 **Preisuntergrenze der KI.** Angehoben auf das 1,2-Fache (vorsichtige Firmen) bis
    1,1-Fache (aggressive) der Vollkosten, statt 1,15 bis 1,05. Im Wettbewerb liegen die
    Preise an dieser Grenze; mit den alten Werten fielen Grundstoffe bis 1929 auf etwa die
    Hälfte des Richtpreises. Gegenläufig: Je höher die Grenze, desto mehr verdient eine
    Werkstatt ohne Entscheidungen (1900 etwa zwei Drittel ihres Startkapitals).
35. 🟡 **Fließband für Motor und Fahrgestell.** Neue Rezepte mit der Technologie
    Fließband (1913) und eigenen Anlagen: etwa ein Sechstel der Arbeitsstunden der
    Werkstattfertigung, wie bei Ford. Ohne sie blieb das Auto bis 1930 ein Luxusgut
    (rund 4.000 Autos im Jahr im Marktmaßstab 0,1); mit ihnen fallen die Preise auf etwa
    die Hälfte des Richtpreises, und die Nachfrage wächst um ein Vielfaches.
36. 🟡 **Förderung im Lauf der Zeit.** Getreide, Baumwolle und Holz haben einen
    Förderindex nach der historischen Weltproduktion (Getreide 1930 das 1,45-Fache von
    1900, Baumwolle 1,65 nach der Baumwollernte 3,6 → 6,0 Mio. t, Holz 1,3). Weitere
    Rohstoffe wachsen über neue Lagerstätten (Entdeckungsjahr). Kleidung: 5 statt 6 Stück
    je Kopf und Jahr bei voller Kaufneigung – der Faserbedarf lag 1900 sonst bei 6,6 statt
    etwa 5 Mio. t, und Baumwolle kostete bis 1929 das Drei- bis Vierfache.
37. 🟡 **Sonstige Gummiwaren.** Gummi hat eine Staatsnachfrage als Stellvertreter für
    Treibriemen, Schläuche, Dichtungen und Isolierungen (etwa ein Drittel des
    Kautschuks um 1900). Ohne sie brauchten nur Reifen Kautschuk, und sein Preis fiel auf
    ein Drittel des Richtpreises.
38. ❓ **Bekannte Restpunkte der Plausibilität** (Protokoll 1900–1930 mit 100 KI-Firmen,
    siehe `docs/FORTSCHRITT.md`):
    - Die Stahlkette (Blech, Draht, Weißblech, Schrauben, Muttern, Nägel) liegt im
      Wettbewerb bei 0,45–0,65 × Richtpreis: Ihre Rezepte haben zu Richtpreisen 26–45 %
      Marge, und die Preise sinken bis an die Untergrenze (1,1–1,2 × Vollkosten). Real
      fielen Stahl- und Nagelpreise 1900–1929 (inflationsbereinigt) ebenfalls um ein bis
      zwei Drittel. Vorschlag: so lassen; sonst das Margenband auf höchstens 30 % enger
      fassen und 17 Rezepte anpassen.
    - Die Werkstatt ohne Entscheidungen verdient 1900–1903 bis zur Hälfte ihres
      Startkapitals im Jahr (Grenze im Protokoll: 50 %). Vorschlag: kleinere
      Startwerkstatt oder Zinsen auf das gebundene Kapital in den Vollkosten.
    - Gummi ist 1902–1911 knapp (Kautschuk bis 2 × Richtpreis), bis die Plantagen in
      Malaya liefern – historisch richtig (Kautschukboom 1910).
    - Das Auto bleibt kleiner als in Wirklichkeit: 1929 etwa 0,7 Mio. statt 5 Mio. Autos
      im Jahr (hochgerechnet). ✅ Benzin gelöst mit M22 (Punkt 42): Untergrenze nach dem
      Brennwert, übrige Verwendung als Nachfrage, Crackanlagen ab 1925.
    - Die Kautschukförderung fällt 1903–1911 auf 40–75 % der Kapazität, obwohl Kautschuk
      das Zwei- bis Dreifache des Richtpreises kostet (schon vor M22). Gummi ist dadurch
      knapper als nötig. Ursache noch nicht gefunden; Vorschlag: bei der nächsten
      Balance-Runde prüfen.
39. 🟡 **Pacht und Förderabgaben (Nacharbeit zu M16).** Rohstoffe zahlen je geförderter
    Einheit einen Anteil ihres Richtpreises im Land (`pacht_anteil`): Baumwolle 0,5,
    Getreide und Rohöl 0,45, Kautschuk 0,4, Kupfererz 0,35, Holz 0,2, Eisenerz 0,15,
    Kohle 0. Bodenrente, Förderzins und Konzessionsabgaben fehlten in den Kosten; wo keine
    Lagerstätte knapp war, fielen die Rohstoffe auf ein Drittel ihres Richtpreises und
    zogen ganze Ketten mit. Gebucht als neue Kostenart „Pacht und Förderabgaben“.
40. 🟡 **Langsamere Preisschritte.** Automatische Preise steigen je Tag um 0,25 % (vorher
    2 %) und sinken um 0,125 % (vorher 1 %), also etwa +8 % und −4 % im Monat. Die
    schnellen Schritte schaukelten Getreide und Mehl im ersten Jahr zwischen 0,55 und
    3,9 × Richtpreis auf; im Weltlauf ergaben die langsamen in allen Prüfungen die
    wenigsten Verstöße. Gegenläufig: Nach einem echten Engpass (Krieg) dauert es Monate,
    bis die Preise ihn zeigen.
41. 🟡 **Ruhigere Märkte (Nacharbeit zu M16).** (a) Ein ausverkaufter Anbieter erhöht
    seinen Preis nur, wenn ein Käufer ohne Ware mehr gezahlt hätte – kauft der Staat
    über seiner Preisgrenze nicht, ist das kein Mangel. (b) Die Startbesetzung beginnt im
    Gleichgewicht (Absatz im Vormonat = geplante Erzeugung). (c) Die KI ändert die
    Auslastung je Entscheidung höchstens um 0,3. (d) Märkte ohne Verkäufe starten beim
    Richtpreis im Land. Erprobt und verworfen: Niederstwertprinzip für die
    Preisuntergrenze, Ausbau erst über der Normalauslastung (beides ohne Verbesserung im
    Weltlauf).

42. 🟡 **Nebenprodukte und neue Verfahren (M22).** (a) Die KI verkauft Nebenprodukte
    nicht unter ihrem Brennwert (Wärme zum Preis des billigsten anderen Brennstoffs im
    Land; Benzin rund 0,1 × Richtpreis). (b) Benzin hat eine übrige Verwendung als
    Staatsnachfrage (0,15 t je Mio. USD BIP; Lösungsmittel, Kocher, Motoren, Heer).
    (c) KI-Firmen steigen mit allen Verfahren, die sie kennen, in neue Produkte ein,
    nicht nur mit gemeinfreien. (d) Nebenprodukte zählen als Erzeugung, Vorprodukte auf
    Halde nicht als Engpass. Ergebnis: Benzin 1929 bei 1,18 × Richtpreis statt 0,00,
    Crackanlagen ab 1925 (Formeln: `docs/FORMELN.md`, M22).
43. 🟡 **Umstellungskurse bei Währungswechseln (M28).** Die Meldung zur Umstellung nennt
    den gesetzlichen Kurs, wo er in den Daten steht (137 Wechsel, darunter alle
    Euro-Einführungen), sonst das Verhältnis der Wechselkurse. Wo die Daten eine
    Zwischenwährung auslassen, weicht dieses Verhältnis vom damaligen Umstellungskurs ab
    (Brasilien: Cruzado Novo 1989 und Cruzeiro Real 1993 fehlen, daher 1990
    „1 Cr$ = 41,2 Cz$“; ähnlich Kroatien 1994 und die Übergangscoupons in Georgien,
    Moldau und Usbekistan). Vorschlag: diese Zwischenwährungen bei Gelegenheit in
    `data/waehrungen/` ergänzen.

## E Vorschläge für die nächsten Meilensteine (05.10.2026)

Auftrag: „Wenn du mit meinen letzten Anforderungen durch bist, überlege, wie du das Spiel
besser machen kannst.“ ✅ Alle acht Punkte sind freigegeben (05.10.2026, Lastenheft
§18.3) und als M22–M29 umgesetzt (Stand und eigenständige Entscheidungen:
`docs/FORTSCHRITT.md`).

Grundlage ist ein Weltlauf 1900–1930 mit Stand M21 (100 KI-Firmen, Seed 1; Protokoll mit
`wsim run --ki 100 --bis 1930-01-01 --protokoll <Ordner>`). Die Prüfungen stehen wie nach
der Nacharbeit zu M16: Versorgung weltweit 18 Verstöße, Preis gegen Richtpreis 146,
keine Pleitewellen. Dazu kamen diese Befunde:

- **Überkapazität:** 30 der 47 Produkte haben zehn Jahre oder länger Überkapazität
  (mehr als das Doppelte des Bedarfs; Kupfererz 1929 das 5,6-Fache). Viele Metallwaren
  kosten 1929 nur 0,45 bis 0,6 des Richtpreises. Blech wird 27 Jahre mit Verlust
  hergestellt.
  - Ursache: Anlagen werden gebaut, aber nie stillgelegt oder verkauft. Dafür gibt es
    keinen Befehl, auch nicht für den Spieler.
- **Einzelne Märkte:** Benzin kostet ab 1913 nichts (0,00 des Richtpreises). Gummi
  deckt 1902–1911 zeitweise nur 26 % der Nachfrage, Autoreifen 1923–1924 nur 74 %.
- **Passive Werkstatt:** Ohne Entscheidungen verdient sie 1900 noch 72 % ihres
  Startkapitals, ab 1920 rund 10 % im Jahr und 1929 nichts mehr. Wer nicht handelt,
  fällt zurück.
- **Rechenzeit:** 30 Jahre dauern 6 Minuten (Release, ein Kern). Im Browser braucht eine
  Monatsrunde mit 100 KI-Firmen rund 1,9 s.

1. ✅ **Anlagen stilllegen und verkaufen (Spieler und KI).**
   - Neue Befehle: eine Anlage vorübergehend stilllegen (geringe Fixkosten, später
     wieder anfahren) oder verkaufen bzw. abreißen (Restwert mit Abschlag).
   - Die KI legt Anlagen still, die über längere Zeit Verlust machen. Das senkt die
     Überkapazität und hebt die Preise wieder in Richtung Richtpreis.
   - Der Spieler kann so Fehlentscheidungen korrigieren und Geld aus schwachen
     Standorten ziehen.
   - Danach eine Balance-Runde für Benzin, Gummi und Reifen.
2. ✅ **Etappenziele nach der Einführung.** Die Einführung endet mit dem ersten Verkauf;
   danach fehlt eine Richtung.
   - Abwählbare Etappen in der Übersicht, jeweils mit Hinweis, wie man sie erreicht
     („Nächste Etappe: erster Monat mit Gewinn – so geht's …“).
   - Beispiele: erster Gewinnmonat, zweite Anlage, eigenes Vorprodukt, zweites Land,
     erste eigene Forschung, Marktführer in einem Land, Eigenkapital verdoppelt.
   - Erreichte Etappen erscheinen als Erfolg im Rundenbericht.
   - Etappen als Daten, die Bewertung als Sicht im Kern, ohne Wirkung auf die Simulation.
3. ✅ **Wettbewerb und Preise im Verlauf (§13.2, §14.2).**
   - Rundenbericht: Preissenkungen der Konkurrenz, neue und ausgeschiedene Anbieter in
     den eigenen Märkten.
   - Markt: Preisverlauf je Produkt und Land über 24 Monate, dazu der eigene
     Marktanteil im Verlauf.
   - Braucht eine kleine Monatsreihe je Markt im Spielstand (neue Formatversion).
4. ✅ **Ansicht Produktionsketten (§14.1).** Je Endprodukt die Kette bis zu den
   Rohstoffen:
   - Verfahren und Anlage je Stufe.
   - Richtpreis gegen geschätzte Stückkosten im Land des Firmensitzes.
   - Was der Spieler selbst herstellt oder kauft, und wo es hakt.
   - Beantwortet „Was lohnt sich als Nächstes, und was brauche ich dafür?“.
5. ✅ **Weiterlaufen bis …** Mehrere Runden am Stück, bis zum Jahresende oder bis zur
   nächsten Warnung bzw. zum nächsten Weltereignis. Spart Klicks, solange alles läuft.
6. ✅ **Ursachen erklären (§14.3).** Tooltips zerlegen einen Wert in seine Teile:
   - Marktpreis aus Richtpreis und Knappheit.
   - Stückkosten aus Material, Personal, Energie und Anlage.
   - Nachfrage aus Bevölkerung, Einkommen und Sättigung.
7. ✅ **Geschichte erzählen.**
   - Währungsreformen als Meldung im Rundenbericht („Ab Dezember 1923 rechnet
     Deutschland in Reichsmark, 1 RM = 1 Billion Mark“).
   - Weltereignisse 1940–2026 als Meldungen; ihre Wirkungen folgen weiter mit Stufe 4.
8. ✅ **Rang in der Übersicht.** Platz der eigenen Firma unter allen Firmen nach
   Eigenkapital und Umsatz, mit der Veränderung zum Vorjahr.

## F Kaufangebote zwischen Firmen (Anfrage vom 05.10.2026)

Auftrag: „Es soll die Möglichkeit geben, dass Wettbewerber lohnenswerte Werke, Bereiche,
Technologien oder Labore abkaufen wollen. Bspw. weil die besonders qualifiziert oder
wirtschaftlich sind. Oder als Vermeidung von Wettbewerb.“

✅ Entschieden (Lastenheft §18.4): Standorte und Labore, Technologien als Lizenz und ganze
Bereiche; alle Firmen untereinander nach denselben Regeln; Annehmen, Ablehnen oder
Gegenangebot. Umsetzung als M30 (Standorte, Labore, Lizenzen) und M31 (Bereiche).

🟡 Ausgestaltung (Vorschlag gilt vorläufig, Formeln in `docs/FORMELN.md`, M30):

1. **Grundwert eines Standorts:** höherer Wert aus Ertragswert (Jahresergebnis × 5) und
   Restwert der Anlagen, dazu Anlagen im Bau und Lager. Ein Standort lässt sich erst ein
   Jahr nach der Gründung kaufen (sonst fehlt ein Ergebnis zum Bewerten).
2. **Motive der KI:** Wettbewerb im selben Markt (je nach Aggressivität bis +60 %),
   qualifizierte Belegschaft, wo sie knapp ist (bis +30 %), eigenes Geschäft (Bauzeit
   gespart, +15 %), Neubau wäre teurer (Hälfte der Ersparnis), Lizenz statt eigener
   Forschung (30–60 % der ersparten Kosten).
3. **Firmenwert:** Was der Käufer über den Buchwerten zahlt, steht als Firmenwert in der
   Bilanz und wird über zehn Jahre abgeschrieben (wie HGB § 253).
4. **Häufigkeit:** Jede KI-Firma prüft monatlich mit 2–6 % Wahrscheinlichkeit ein
   Geschäft; an den Spieler geht höchstens ein neues Angebot je Monat.
5. **Selbst gebraucht:** Ein Kraftwerk, dessen Strom die eigenen Werke im Land brauchen,
   und das einzige Labor, solange es forscht, gibt eine KI-Firma nur zum Neubaupreis ab.
   Kraftwerke kauft sie nur, wo ihr selbst Strom fehlt, Labore nur, wenn sie keines hat
   (Strom lässt sich nicht handeln; ein zweites Labor nutzt die KI nicht).
6. **Bereiche (M31):** alle Standorte einer Warengruppe – jeweils ganz – mit der Marke.
   Die Marke ist so viel wert wie die Werbung für dieselbe Bekanntheit; der Käufer
   übernimmt die höhere der beiden Bekanntheiten je Land. Umfasst ein Bereich die ganze
   Erzeugung einer KI-Firma, verlangt sie den Kernaufschlag (wie für den einzigen
   Standort mit Anlagen).
7. **Später:** Übernahme ganzer Firmen über Anteile (Stufe 3), Kartellaufsicht gegen
   Aufkäufe zur Wettbewerbsvermeidung (Stufe 4, Regulierung), Patente (Stufe 5).

## G Produkte bis 2026 (Entscheidung vom 05.10.2026)

✅ Entschieden (Lastenheft §18.4): Die Produktbreite aus Stufe 5 wird Epoche für Epoche
vorgezogen. 🟡 Vorschlag für die Meilensteine (Auswahl je Epoche mit Recherche zu Rezept,
Anlage, Erfindungsjahr, Richtpreis und Nachfrage; Regeln aus Lastenheft §17.2):

| Meilenstein | Epoche | Beispiele für neue Ketten und Produkte |
| --- | --- | --- |
| M32 | 1915–1939 | Aluminium (Bauxit), Kunstseide, Phenolharz, Röhrenradio, Kühlschrank, Staubsauger, Lastwagen, Traktor, Verkehrsflugzeug, Stickstoffdünger |
| M33 | 1940–1964 | Erdöl-Kunststoffe, Polyamid, Fernseher, Waschmaschine, Transistorradio, Düsenflugzeug, Antibiotika |
| M39 | 1965–1989 | Halbleiter und Mikrochip, Taschenrechner, Farbfernseher, Mikrowelle, Personal Computer, Videorekorder, CD-Spieler |
| M40 | 1990–2009 | Mobiltelefon, Laptop, Digitalkamera, Lithium-Ionen-Akku, Flachbildfernseher |
| M41 | 2010–2026 | Smartphone, Tablet, Elektroauto, Solarmodul, Windkraftanlage, Wärmepumpe, LED-Lampe |

Je Epoche: neue Rohstoffe mit Lagerstätten, Technologien mit Erfindungsjahr, Verdrängung
älterer Produkte (z. B. Transistorradio verdrängt Röhrenradio, LED die Glühlampe), Texte,
Prüfung im Weltlauf. Rüstungsgüter bleiben bei Stufe 4.

✅ **M32 (1915–1939) umgesetzt** wie in der Tabelle, dazu Zellstoff, Kleinmotor, Flugmotor,
Ammoniak und Chilesalpeter als Vorprodukte (Einzelheiten: `docs/FORTSCHRITT.md`, M32).
🟡 Vorläufig entschieden:

1. **Nachfrage erst ab Verfügbarkeit:** Nach einem Produkt fragt erst jemand, wenn es
   sich herstellen lässt (Erfindungsjahr erreicht oder von einer Firma vorzeitig
   erforscht). Vorher gab es Nachfrage nach Röhrenradios, die niemand bauen konnte.
2. **Käufer von Lastwagen, Traktoren, Verkehrsflugzeugen und Dünger** sind in Stufe 1 die
   Staaten (Bedarf je BIP); Speditionen, Höfe und Fluggesellschaften als eigene Kunden
   kommen mit späteren Stufen. Der Düngerbedarf entspricht 1913; sein starker Anstieg
   danach folgt mit den späteren Epochen.
3. **Aluminiumhütten** stehen am eigenen Wasserkraftwerk (in der Investition); Netzstrom
   wäre für 20 MWh je t unbezahlbar.
4. **Anlagengrößen** der neuen Ketten sind einzelne Linien (Tonerde 30 t, Hütte 10 t,
   Flugmotoren 0,5 Stück je Tag), damit sie zu den kleinen Märkten passen; große Werke
   bestehen aus mehreren Einheiten.
5. **KI für neue Produkte** (alle Firmen gleich, `docs/FORMELN.md`, M32): Pioniere bauen,
   was noch niemand herstellt; Marktlücken werden erforscht; Anlagen im Bau zählen gegen
   die offene Nachfrage; Engpässe mit Ausweichen; Vorprodukte entstehen dort, wo die
   Fracht am geringsten ist.
6. **Bekannte Grenzen:** Verkehrsflugzeuge sind 1925–1939 nur zu 48–86 % versorgt (wenige,
   teure Stücke: Händler verteilen Bruchteile auf viele Länder, der Preis liegt nahe der
   Obergrenze des Staats); in den ersten zwei bis vier Jahren eines neuen Produkts fehlt
   Ware, bis die ersten Werke fertig sind. Die Aluminiumhütten wurden für den Anlauf der
   Kochtopf-Nachfrage gebaut und stehen danach großteils still (Verlust); einzelne
   Anlagen je Standort legt die KI nicht still (Regel aus M22).

✅ **M33 (1940–1964) umgesetzt** wie in der Tabelle, dazu Ethylen, Polyethylen,
Bildröhre, Germanium, Transistor, Strahltriebwerk und Synthesekautschuk (Einzelheiten:
`docs/FORTSCHRITT.md`, M33). 🟡 Vorläufig entschieden:

1. **Einführungsjahre** sind die der Serienfertigung für Verbraucher: Fernseher 1946
   (Regelbetrieb gab es ab 1935/36, in Serie gebaute Geräte erst nach dem Krieg),
   Waschvollautomat 1937, Transistorradio 1954, Düsenverkehrsflugzeug 1952, Penicillin
   1943 (Tiefentank-Fermentation).
2. **Penicillin** ist ein Verbrauchsgut des Grundbedarfs (0,5 Behandlungen je Kopf und
   Jahr); **Kunststoffwaren** sind ein Sammelgut in kg (Eimer, Schüsseln, Folien).
3. **Germanium** kommt aus eigenen Lagerstätten (Tsumeb, Kipushi, Tri-State); in
   Wirklichkeit fiel es in Zink- und Kupferhütten an, Zink gibt es im Spiel noch nicht.
4. **Verdrängung beim Staatsbedarf:** Das Düsenflugzeug ersetzt das Verkehrsflugzeug über
   15 Jahre (`verdraengung_staat_jahre`), gerechnet ab dem Jahr, in dem es sich bauen ließ.
5. **Zweite Wege:** Kautschuk auch synthetisch aus Ethylen und Benzin (1937), Polyamid auch
   aus Kohle (Teer der Kokereien). Für die Ebenen des Produktbaums zählt bei mehreren
   Rezepten der einfachste Weg.
6. **KI** (alle Firmen gleich, `docs/FORMELN.md`, M33): Verfahrenswahl mit Knappheit,
   Einstieg in teure Märkte samt Forschung für deren Verfahren, Gruben passend zur
   Konzession, neue Konzessionen nur mit Restvorrat für 10 Jahre, nur Lager von Verkäufern
   zählen als Halde.
7. **Förderung übers Jahr verteilt:** Eine Konzession darf bis zu einem Tag höchstens den
   bisherigen Anteil ihrer Jahresmenge fördern (vorher förderte sie die Jahresmenge in
   wenigen Wochen und stand dann still).
8. **Bekannte Grenzen:** Kupfererz ist ab 1954 knapp (1,8- bis 2,4-facher Richtpreis, ein
   bis zwei Förderer); Kupferdraht für den Staat fällt dann zeitweise aus. Düsenflugzeuge
   baut bis 1964 nur eine Firma. Vorschlag: neue Kupferlagerstätten und ein Förderindex
   für Erze mit den Epochen ab 1965 (M39).

✅ **M39 (1965–1989) umgesetzt** wie in der Tabelle, dazu Quarz, Reinstsilizium,
Farbbildröhre und Magnetron als Vorprodukte; Transistoren auch aus Silizium (Einzelheiten:
`docs/FORTSCHRITT.md`, M39). 🟡 Vorläufig entschieden:

1. **Mikrochip als Sammelgut:** ein Durchschnittschip (Prozessor, Speicher, Logik) um
   1980; Taschenrechner brauchen 2, CD-Spieler 10, Videorekorder 15, Personal Computer
   150. Den Preisverfall der Chips bildet die Weiterentwicklung (M37) nur gedämpft ab.
2. **Personal Computer** zählen samt Monitor; Heimcomputer am Fernseher sind im Mittel
   enthalten.
3. **Rohstoffe für die Epoche** (Vorschlag aus M33): Eisenerz, Kupfererz und Erdöl
   bekommen die großen Funde nach 1950 mit Entdeckungsjahr (12 Eisenerz-, 13 Kupfererz-,
   16 Ölfelder) und Eisen- und Kupfererz einen Förderindex. Bis M38 waren 1964 alle
   Eisenerzgruben ausgelastet; nach 1965 kostete Erz das Vierfache, Kupfererz das
   2,5-Fache des Richtpreises.
4. **Dünger wächst schneller als das BIP:** `staatsnachfrage.verlauf` (neu) gibt dem
   Staatsbedarf einen Verlauf über die Jahre; beim Stickstoffdünger folgt er dem
   Verbrauch je BIP (1913 = 1, 1970 = 8, 1980–1990 = 10, 2020 = 5). Dasselbe Feld dient
   ab M41 für Windkraftanlagen und Solarmodule.
5. **KI bereitet kommende Produkte vor:** Produkte, die sich historisch in fünf Jahren
   herstellen lassen, zählen schon als Marktlücke für die Forschung
   (`forschung_vorlauf_jahre`). Vorher gab es Taschenrechner (1971) im Weltlauf erst ab
   1975 oder bis 1976 gar nicht, Personal Computer (1977) und Videorekorder (1976) 1978
   noch nicht, weil ihre Voraussetzungen (Reinstsilizium, Planartechnik, integrierte
   Schaltung, Mikroprozessor) erst mit der ersten Nachfrage erforscht wurden.
6. **Bekannte Grenzen:** Neue Elektronik ist drei bis sechs Jahre nach dem Erfindungsjahr
   voll versorgt (Taschenrechner 1973–1977, Personal Computer 1983, CD-Spieler
   1987–1989); Holz wird in einem der Weltläufe ab 1986 teuer.

✅ **M40 (1990–2009) umgesetzt:** Mobiltelefon, Laptop, Digitalkamera,
Lithium-Ionen-Akku (mit Lithium und Kobalt), LCD-Panel, Flachbildfernseher und
DVD-Spieler (Einzelheiten: `docs/FORTSCHRITT.md`, M40). 🟡 Vorläufig entschieden:

1. **LCD-Panel in m² Bildfläche** (neue Einheit `m2`), so passen Laptop, Mobiltelefon,
   Digitalkamera und Fernseher mit ihren Bildgrößen an dasselbe Vorprodukt.
2. **Chipgehäuse aus Glas und Keramik** statt Phenolharz: Phenolharz war in den
   Chipländern knapp, und nach der Ausbauregel baute niemand mehr aus.
3. **Holz und Baumwolle:** Der Förderindex folgt bei Baumwolle der Welternte (2010 das
   6,9-Fache von 1900); bei Holz kommt ab 1980 ein Zuschlag für Altpapier,
   Sägenebenprodukte und Plantagenholz hinzu, die das Spiel nicht eigens abbildet.
4. ✅ (C1: Arbitrage der Händler, `docs/FORMELN.md`) **Bekannte Grenze, mit Vorschlag:** Neue Elektronik bleibt in einzelnen Ländern lange
   beim 2,5- bis 3-Fachen des Richtpreises, obwohl sie anderswo billig zu haben ist
   (2009: LCD-Panel in den USA 11 000 USD je m², in China 3 000–5 800 USD; 25 Firmen
   kennen das Verfahren, drei bauen). Grund: Händler beliefern nur Märkte mit offener
   Nachfrage. Ist ein Land zu hohen Preisen versorgt, kommt keine billigere Ware herein.
   🟡 Vorschlag: Händler kaufen auch dann im Ausland, wenn die Ware dort samt Fracht und
   Händlerspanne deutlich billiger ist als im Land (Arbitrage), bis sich die Preise auf
   den Frachtabstand angleichen. Das ändert alle Märkte und braucht Weltläufe über alle
   Epochen; deshalb erst nach Freigabe.

✅ **M41 (2010–2026) umgesetzt:** Smartphone, Tablet, Elektroauto (mit Traktionsbatterie
und Elektroantrieb), Solarmodul (mit Roh- und Solarsilizium), Windkraftanlage, Wärmepumpe
und LED-Lampe (Einzelheiten: `docs/FORMELN.md`, M41; `docs/FORTSCHRITT.md`, M41).
🟡 Vorläufig entschieden:

1. **Neue Einheiten** `kwh` (Traktionsbatterie) und `kwp` (Solarmodul): Fahrzeuge und
   Module brauchen sehr verschiedene Mengen desselben Vorprodukts.
2. **Rohsilizium vom Staatsmarkt:** Sonst wären etwa 0,5 Mio. t Quarz im Jahr aus den
   kleinen Quarzbrüchen nötig. Solarsilizium ist ein eigenes Produkt, damit die
   Solarnachfrage nicht in die Chips greift.
3. **Batterie mit Kobalt und Graphit:** Im ersten Weltlauf entstand kein Batteriewerk,
   weil Kobalt und Kohle knapp und alle ihre Konzessionen vergeben waren. Statt die
   Batterie ohne diese Stoffe zu bauen (Vorschlag des Hilfsagenten), baut die KI ein
   Produkt, das noch niemand herstellt, jetzt auch bei knappen Vorprodukten (Kern, M41).
4. **Rohstoffe bis 2026:** Förderindex für Kohle (neu), Eisenerz, Kupfererz und Kobalt
   nach der Welterzeugung; die Kobaltgewinnung braucht ein Viertel der Stunden.
5. **Bewusst weggelassen:** Reifen am Elektroauto, Aluminiumrahmen am Solarmodul,
   Nickel, Mangan, Elektrolyt und Separator der Zellen (höchstens vier Vorprodukte).
6. **Späte Startjahre:** Ein Vorprodukt, das zum Start zu wenig gebraucht wird, um eine
   mittlere Anlage zu füllen, bekommt in der Startbesetzung jetzt eine kleine Anlage.
   Vorher fehlte es, und alle Stufen darüber fielen weg (Start ab 1920 ohne Aluminium,
   ab 1970 ohne Transistoren, ab 2000 ohne Akkus, Mobiltelefone und Laptops).
7. **Bekannte Grenzen (Weltläufe bis 2026):** Neue Märkte haben lange nur ein bis drei
   Hersteller und kosten bis zum Vierfachen des Richtpreises (Smartphone in einem Lauf bis
   2018 nur zu 4 % versorgt); Solarmodule und Windkraftanlagen bleiben bis etwa 2020 knapp.
   Das Mobiltelefon wird vom Smartphone nur teilweise verdrängt.
8. ✅ **Rohstoffvorräte ab 2018** (umgesetzt, Weltläufe in `docs/FORTSCHRITT.md`, M41): Die abbauwürdigen Vorräte
   einer Lagerstätte wachsen mit dem Förderindex ihres Rohstoffs, wie in der Wirklichkeit
   Erkundung und Technik die Reserven trotz Förderung wachsen ließen. Bisher waren bis 2026
   zwölf von 26 Eisenerz-Lagerstätten und die größten Ölfelder leer. Für Baumwolle und
   Getreide steigt der Förderindex nach 2010 weiter (Ersatz für Polyester und
   Ertragssteigerung, die das Spiel nicht eigens abbildet).

9. ✅ (C2: Förderkurve und Ausbau an allen Standorten, C1: Arbitrage; `docs/FORMELN.md`) **Rohöl und Baumwolle nach 2015** (vorher 🟡 Vorschlag, offen): Die Vorräte reichen jetzt,
   trotzdem kosten Rohöl (in einem von zwei Läufen ab 2023) und Baumwolle (2005–2025) das
   Zwei- bis Vierfache. Ursachen aus dem Spielstand 2026: (a) Ölfelder fördern bis zuletzt
   mit voller Leistung und sind dann schlagartig leer; neue Felder erschließt die KI nur
   zögerlich. (b) Eine Firma baut je Quartal nur am besten Standort aus, um ein Viertel –
   bei zwölf Plantagen einer Firma dauert das Jahrzehnte. (c) US-Plantagen sind zu heutigen
   Löhnen kaum rentabel. Vorschlag: Förderkurve (nach der Hälfte des Vorrats sinkt die
   zulässige Förderung mit dem Restvorrat), Ausbau an allen Standorten eines Produkts mit
   hoher Marge im selben Quartal, und der Arbitragehandel aus M40, Punkt 4 (Preisinseln).
   Ändert alle Weltläufe; erst nach Freigabe.

## H Grundstücke und Werksgrößen (Anfrage vom 06.10.2026)

Auftrag: „Es sollte verschiedene Größen von Werken geben. In jedem Land stehen nur
begrenzt Grundstücke zur Verfügung. Diese sind auch noch verschieden groß. Wenn auf den
Grundstücken Gebäude ausgebaut werden, dann kann es sein, dass sie an die
Grundstücksgröße kommen und nicht erweiterbar sind. In diesem Fall muss in dem Land ein
neues Grundstück angelegt werden. […] Bei den Grundstücken und den Größen soll es schon
einen kleinen Wettbewerb um gute Grundstücke geben. Man soll Grundstücke kaufen und/oder
pachten können.“

✅ Entschieden am 06.10.2026: Grundstücke mit Größe, Preis **und Lage** gleich im ersten
Schritt; Anlagen in **fünf Größenklassen**. 🟡 Ausgestaltung (erster eigener Maßstab;
Zahlen werden im Weltlauf nachgestellt):

1. **Grundstücke je Land.** Jedes Land (bzw. jede Region, Abschnitt I) bietet eine Liste
   freier Gewerbegrundstücke an. Jeder Standort – Werk, Kraftwerk, Lager, Niederlassung,
   Forschungszentrum – steht auf genau einem Grundstück. Förderstätten bleiben an ihre
   Konzession gebunden; die Konzession ist ihr Grundstück.
2. **Fläche der Anlagen.** Jede Anlage braucht Fläche: als Regel 1 ha je 10 Mio. USD
   Investition (mindestens 0,2 ha), dazu 20 % für Wege, Lager und Verwaltung. Flächenhungrige
   Anlagen bekommen in den Daten einen eigenen Wert (`flaeche_ha`: Raffinerie, Hüttenwerk,
   Flugzeugwerk, Kraftwerk). Beispiele: 10 Hochöfen um 1900 ≈ 2,4 ha; Großflugzeugwerk
   1960 ≈ 60 ha.
3. **Werksgröße = Grundstück.** Ein Werk wächst mit seinen Anlagen, bis die Fläche des
   Grundstücks belegt ist. Danach baut die Firma im selben Land auf einem neuen Grundstück
   weiter (eigener Standort mit eigenen Löhnen, Lagern und Angeboten). Die KI tut das von
   selbst; der Spieler sieht die belegte und freie Fläche je Standort.
4. **Menge im Spielverlauf.** Die Gewerbefläche eines Landes wächst mit seiner Wirtschaft:

       Fläche(t) = 25 ha je Mrd. USD BIP(t) (Kaufkraft 2026) · Marktmaßstab

   Jedes Jahr am 1. Januar kommen neue Grundstücke hinzu, bis die angebotene Fläche
   (belegt und frei) diesen Wert erreicht; aufgegebene Standorte geben ihr Grundstück
   zurück. Beispiele bei 100 KI-Firmen (Maßstab 0,1):

   | Land | 1900 | 1960 | 2026 |
   | --- | --- | --- | --- |
   | Deutschland | 914 ha ≈ 150 Grundstücke | 3 633 ha ≈ 200 | 18 433 ha ≈ 580 |
   | USA | 2 750 ha ≈ 450 | 12 375 ha ≈ 680 | 94 028 ha ≈ 2 970 |
   | Brasilien | 70 ha ≈ 11 | 989 ha ≈ 55 | 15 119 ha ≈ 480 |
   | Baltikum | 55 ha ≈ 9 | 209 ha ≈ 11 | 944 ha ≈ 30 |
   | Mittelafrika | 10 ha ≈ 2 | 43 ha ≈ 2 | 639 ha ≈ 20 |

5. **Größen.** Neue Grundstücke kommen in vier Klassen; die Flächen wachsen mit der Zeit
   (Industriegebiete werden größer) um den Faktor 1 + (Jahr − 1900) / 30 (1900: 1, 1960: 3,
   2026: 5,2):

   | Klasse | Fläche um 1900 | Anteil reich / mittel / arm |
   | --- | --- | --- |
   | klein | 0,5–2 ha | 40 / 55 / 70 % |
   | mittel | 2–6 ha | 35 / 30 / 25 % |
   | groß | 6–20 ha | 18 / 12 / 5 % |
   | sehr groß | 20–60 ha | 7 / 3 / 0 % |

   „Reich“ ab 15 000 USD BIP je Kopf, „arm“ unter 5 000 USD.
6. **Preis und Wettbewerb.** Bodenpreis je ha = 200 000 USD · Preisniveau des Landes ·
   (1 + 2 · belegter Anteil der Gewerbefläche). Wer zuerst kommt, nimmt die großen und
   günstigen Grundstücke; danach bleiben kleine und teure. Die KI wählt das günstigste
   Grundstück (je ha), das ihren Bedarf plus 50 % Reserve fasst, sonst das größte freie.
7. **Kaufen oder pachten.** Kauf: Bodenpreis sofort, als „Grundstücke“ im
   Anlagevermögen ohne Abschreibung; beim Verkauf des Standorts geht das Grundstück mit.
   Pacht: jährlich 5 % des aktuellen Bodenwerts (Kostenart Pacht), Laufzeit 30 Jahre mit
   Verlängerung; keine Kapitalbindung. Die KI pachtet, wenn ihr Kassenstand den Kauf nicht
   trägt. Die bisherigen Standortkosten bleiben als Gebäude- und Erschließungskosten.
8. **Lage:** Jedes Grundstück liegt in der Stadt, am Hafen (nur Länder mit Küste) oder
   auf dem Land. Stadt: teurer Boden, mehr und besser ausgebildete Arbeitskräfte, kleinere
   Grundstücke. Hafen: billigere Fracht über See für alles, was der Standort ein- und
   verkauft. Land: billiger Boden, große Grundstücke, weniger Arbeitskräfte und längere
   Wege zum Kunden. Die genauen Faktoren folgen mit den Formeln (`docs/FORMELN.md`).
9. **Anlagengrößen:** Jede Anlage gibt es in fünf Größen – sehr klein, klein, mittel,
   groß, sehr groß (Kapazität 0,25 / 0,5 / 1 / 2 / 4 der Datenwerte). Größere Anlagen
   kosten je Kapazität weniger Investition (Investition wächst mit Kapazität^0,7) und
   weniger Arbeit je Einheit (Kapazität^−0,15), brauchen aber mehr Fläche, Kapital und
   Absatz; kleine Anlagen passen in kleine Märkte und auf kleine Grundstücke.
10. **Spielstände:** Bestehende Standorte bekommen beim Laden ein passendes, gekauftes
    Grundstück.

Umgesetzt mit M35 (Punkte 1–8 und 10; Formeln: `docs/FORMELN.md`, Abschnitt M35) und
M36 (Punkt 9; Abschnitt M36). 🟡 Eigenständig entschieden:

1. **Pacht ohne Laufzeit:** Die Pacht läuft, bis der Standort aufgegeben oder das
   Grundstück gekauft wird; sie folgt jeden Monat dem heutigen Bodenwert. Eine Laufzeit
   mit Verlängerung brächte nur eine weitere Frist ohne Entscheidung.
2. **Wahl der KI nach Kosten statt nach Preis je ha:** Unter den passenden Grundstücken
   nimmt die KI das mit den geringsten jährlichen Kosten aus Pacht auf den Wert und
   Lieferkosten auf den erwarteten Umsatz. Nach dem Preis je ha allein hätte sie fast
   nur Grundstücke auf dem Land genommen (im Test 1903: 233 Stadt, 257 Hafen, 60 Land
   statt fast nur Land).
3. **Lagefaktoren:** Stadt Fläche ×0,5, Boden ×2, Anwerben +10 % (wie ein
   Lohnaufschlag); Hafen Fläche ×1, Boden ×1,2, Seefracht ×0,8; Land Fläche ×2,
   Boden ×0,5, Anwerben −10 %, Lieferkosten 2 % des Umsatzes im Land.
4. **Startstandorte:** Die der KI-Firmen stehen auf gekauften Grundstücken (Teil des
   Startkapitals), die Werkstatt oder Niederlassung des Spielers auf einem gepachteten,
   damit die Startkasse bleibt. Fehlt ein passendes Grundstück, entsteht eines (Stadt).
5. **Ohne Wahl** (Befehl `FoundSite`, z. B. in Kopflos-Läufen) nimmt ein Standort das
   größte freie Grundstück. In der Oberfläche wählt der Spieler immer selbst.
6. **Anzeige:** Gründen mit Liste der freien Grundstücke (Filter Lage und Größe), Kauf
   oder Pacht; in der Werksansicht belegte Fläche, Besitz und „Grundstück kaufen“; beim
   Bauen Fläche und „noch Platz für …“; im Länderdetail die Gewerbeflächen; im
   Rundgang der Einführung ein Schritt dazu.
7. **Anlagengrößen (M36):** Fläche und Bauzeit wachsen wie die Investition bzw. mit
   Kapazität^0,3; Strom und Vorprodukte je Stück bleiben gleich. Die KI wählt die größte
   Größe, die ihre gewünschte Kapazität nicht übersteigt, und weicht bei knappem Geld
   oder Platz auf kleinere aus. Startbesetzung und Startformen bleiben mittel, weil die
   Daten darauf abgestimmt sind; alte Spielstände und Befehle meinen mittel.

## I Länder zu Regionen zusammenfassen (Anfrage vom 06.10.2026)

Auftrag: „Du kannst bei den Ländern und Regionen gerne einiges zusammenfassen. Bspw.
Ozeanien […]. In Afrika kannst du gerne ein paar niedrig bevölkerte Länder zu Gruppen
zusammenfassen. Bspw. ‚Mittelafrika‘. Wenn ich am Ende 100 Länder/zusammengefasste
Regionen habe, ist das ein guter Zielwert.“

✅ Entschieden am 06.10.2026 (Umsetzung vor den restlichen Epochen). 🟡 Ausgestaltung:
197 → **111 Einträge**. Kleinstaaten gehen im Nachbarland auf, kleine
Länder bilden Regionen. Bevölkerung, BIP, Fläche, Grenzen und Karte werden addiert; BIP je
Kopf und Gini gewichtet; Hauptstadt und Währung sind die des größten Mitglieds. Eine
Region behält den ISO-Code ihres größten Mitglieds, wenn es mehr als die Hälfte der
Einwohner stellt („Belgien und Luxemburg“), sonst bekommt sie einen frei verfügbaren
ISO-Code X__.

| Kontinent | vorher → nachher | Zusammenfassungen |
| --- | --- | --- |
| Europa | 44 → 26 | Andorra → Spanien; Monaco → Frankreich; San Marino, Malta → Italien; Liechtenstein → Schweiz; Luxemburg → Belgien; Island → Norwegen; Moldau → Rumänien; Slowakei → Tschechien; Zypern → Griechenland; **Baltikum** (EST, LVA, LTU); **Westbalkan** (SRB, HRV, BIH, MNE, MKD, ALB, Kosovo, SVN) |
| Asien | 49 → 33 | Osttimor → Indonesien; Brunei → Malaysia; Bhutan → Nepal; Malediven → Sri Lanka; Laos → Kambodscha; **Kaukasus** (AZE, GEO, ARM); **Zentralasien** (TJK, KGZ, TKM); **Golfstaaten** (ARE, KWT, QAT, BHR, OMN); **Levante** (JOR, LBN, PSE) |
| Afrika | 55 → 32 | Westsahara → Marokko; Südsudan → Sudan; Gambia → Senegal; Mauritius, Komoren, Seychellen → Madagaskar; **Mittelafrika** (TCD, CAF, COG, GAB, GNQ, STP); **Westafrika** (GIN, SLE, LBR, GNB, CPV, MRT); **Togo und Benin**; **Ruanda und Burundi**; **Horn von Afrika** (SOM, DJI, ERI); **Südliches Afrika** (NAM, BWA, LSO, SWZ) |
| Nordamerika | 23 → 7 | **Mittelamerika** (GTM, HND, NIC, SLV, CRI, PAN, BLZ); **Hispaniola** (HTI, DOM); **Karibik** (JAM, TTO, BHS, BRB, LCA, GRD, VCT, ATG, DMA, KNA); USA, Mexiko, Kanada, Kuba bleiben |
| Südamerika | 12 → 10 | **Paraguay und Uruguay**; **Guayana** (GUY, SUR) |
| Ozeanien | 14 → 3 | Australien, Neuseeland, **Ozeanien** (PNG, FJI, SLB, VUT, WSM, KIR, FSM, TON, MHL, PLW, NRU, TUV) |

Folgen: Lagerstätten, Weltereignisse, Namensgruppen, reale Firmen und Währungen verweisen
dann auf die Region; Spielstände mit den alten Ländern werden beim Laden umgerechnet
(Standorte und Märkte gehen in die Region über).

Umgesetzt mit M34 (Formeln: `docs/FORMELN.md`, Abschnitt M34). 🟡 Eigenständig entschieden:

1. **Gini einer Region** als bevölkerungsgewichtetes Mittel (die Ungleichheit zwischen
   den Ländern fehlt; für Regionen aus ähnlich reichen Ländern genügt das).
2. **Währungen**, die nach dem Zusammenfassen kein Land mehr verwendet (75, etwa der
   Luxemburger Franc), entfallen; sie bleiben in der Versionsgeschichte.
3. **Alte Spielstände:** Standorte, Firmensitze und Befehle gehen in die Region über. Bei
   Werten je Land (Märkte) gilt der Eintrag unter dem Code der Region bzw. des
   führenden Landes; die der übrigen Länder entfallen statt addiert zu werden (Preise
   und Bekanntheit lassen sich nicht sinnvoll addieren, Länderwerte werden ohnehin neu
   berechnet).
4. **Anzeige:** Das Länderdetail nennt unter „Umfasst“ die Länder einer Region; die
   Karte zeigt ihren Umriss ohne innere Grenzen.

## J Weiterentwicklung erforschter Produkte (Anfrage vom 06.10.2026)

Auftrag: „Bei der Forschungskette: Wenn ein erforschtes Produkt weiter erforscht wird,
dann soll das betreffende Produkt weiterentwickelt und verbessert werden können. Das gilt
auch für Vormaterialien oder Halbzeuge.“

🟡 Vorschlag:

1. **Entwicklungsstufen je Produkt und Firma.** Kennt eine Firma die Technologien eines
   Produkts, kann ihr Forschungszentrum es weiterentwickeln, Stufe für Stufe (1–5). Das
   gilt für jedes Produkt, auch Rohstoffe, Halbzeuge und Bauteile (besseres Garn,
   reinerer Stahl, sparsamerer Motor).
2. **Wirkung je Stufe:** +4 Qualitätspunkte des Produkts (alle Rezepte der Firma) und
   −3 % Arbeitsstunden sowie −2 % Vorprodukte je Einheit. Bessere Vorprodukte heben die
   Qualität der daraus gebauten Waren (bestehende Regel: +0,3 je Punkt), bessere
   Qualität bringt Marktanteile (Kaufentscheidung, M16).
3. **Aufwand:** Stufe *n* kostet 20 % des Forschungsaufwands der Technologie des Produkts
   × 1,6^(*n* − 1). Nachzügler forschen billiger (wie bei Technologien: −10 % je Jahr
   seit der ersten Firma, höchstens −80 %); Stufen werden nach 15 Jahren Allgemeingut.
4. **KI:** Kompetente Firmen entwickeln ihre umsatzstärksten Produkte weiter, wenn keine
   neue Technologie ansteht. Lizenzen (M30) können auch Entwicklungsstufen umfassen.
5. **Anzeige:** Forschungsansicht mit „Weiterentwickeln“ je Produkt; Produkt- und
   Marktansicht zeigen die Stufe der eigenen Firma und der Wettbewerber.

Umgesetzt mit M37 (Formeln: `docs/FORMELN.md`, Abschnitt M37). 🟡 Eigenständig
entschieden: Grundlage des Aufwands ist der größte Forschungsaufwand der Technologien des
Produkts, mindestens 10 000 Punkte (Produkte ohne erforschte Technologie); Produkte ohne
Technologie (Ernte, Abbau) forschen im Fachgebiet ihrer Branche. Ein Zentrum arbeitet nach
einer Stufe an der nächsten weiter. Die KI entwickelt nur, wenn sich die nächste Stufe in
5 Jahren bezahlt macht (geschätzt 3 % des Umsatzes je Stufe und Jahr). Lizenzen und
Kaufangebote übertragen vorerst keine Stufen.

## K Pleiten: Standorte weitergeben statt aufgeben (aus dem Weltlauf, 06.10.2026)

Beobachtung: Geht eine KI-Firma pleite, werden ihre Konzessionen frei und müssen neu
erschlossen werden (bis zu zwei Jahre). 1955 fiel so ein großer Eisenerzförderer aus; Erz
kostete 1956 das 3,7-Fache des Richtpreises. 🟡 Vorschlag: Der Insolvenzverwalter bietet
die Standorte zuerst anderen Firmen an (Auktion zum halben Grundwert, M30, auch dem
Spieler); was keiner nimmt, wird stillgelegt. Die Erschließung einer Konzession bleibt
erhalten, ein neuer Betreiber muss nur die Anlagen bauen.

Umgesetzt mit M38 (Formeln: `docs/FORMELN.md`, Abschnitt M38). 🟡 Eigenständig
entschieden: Die Versteigerung dauert 30 Tage, das Mindestgebot ist der halbe Grundwert.
Den Zuschlag bekommt das höchste Gebot zum Preis des zweithöchsten; eine KI-Firma bietet
nur für Standorte, die sie auch sonst kaufen würde, und zwar ihr übliches Kaufangebot
(M30), soweit ihre Kasse reicht. Bei Gleichstand gewinnt eine Firma, die im Land schon vertreten
ist. Ein Standort ohne Gebot wird wie bisher aufgegeben (Konzession und Grundstück frei).
Lizenzen und Bereiche einer insolventen Firma werden nicht versteigert.

## L Bedienung und Tests (Anfrage vom 06.10.2026)

Auftrag: „Bitte führe reichlich komplette Testläufe durch. Achte dabei auch auf
Usability. Bring gerne in Eingabefelder auch noch automatische 1.000er-Punkte zur
Trennung ein. Teile unübersichtliche Übersichten gerne nochmal auf. Binde alles
Notwendige in das Tutorial ein.“

Dazu am selben Tag: „Stelle die Browser-Version erstmal zurück. […] Mehr KI-Gegner.
Mache einen Simulationslauf mit 500 Gegnern.“

✅ Entschieden: Die Browser-Version ist vorerst zurückgestellt (lokal keine Prüfung, in der
CI ohne Abbruch). Ein Weltlauf mit 500 KI-Firmen prüft Tempo und Balance bei mehr
Gegnern. Zahlenfelder zeigen beim Tippen Tausenderpunkte (200.000.000, Dezimalkomma
wo nötig); überladene Ansichten werden nach einer Bedienbarkeitsprüfung aufgeteilt; das
Tutorial erklärt Regionen, Grundstücke, Anlagengrößen und die Weiterentwicklung; jeder
Meilenstein endet mit kompletten Testläufen (Weltlauf, Testpartie, Oberflächentests,
Browser-Version).

## M Produktnamen je Firma (Anfrage vom 06.10.2026)

Auftrag: „In den Produkten hast du konkrete Produktnamen eingetragen. Bspw. Tesla oder
Nissan Leaf oder iPhone. Das sollte alles neutralisiert werden in Elektrofahrzeug bspw.
oder Smartphone. Jede Firma vergibt individuelle Produktnamen. Keine echten Produkte.“

✅ Entschieden: Echte Namen sind aus Texten, Weltereignissen und Datenkommentaren entfernt
(`docs/FORTSCHRITT.md`, „Keine echten Produktnamen“). Eigene Produktnamen je Firma kommen
als M42 gleich nach M41 (Antwort vom 06.10.2026: „Ja, gleich nach M41“).

✅ **M42 umgesetzt** (Regeln: `docs/FORMELN.md`, Abschnitt M42). 🟡 Eigenständig
entschieden:

1. **Nur Endprodukte** tragen Namen; Rohstoffe, Halbzeuge, Bauteile und Strom bleiben
   Gattungsware.
2. **Zwei Stile:** Modellnamen für Geräte, Maschinen und Fahrzeuge („Kelvor M80“, bis
   1939 auch „Marvik Typ 12“), Markennamen für Waren des täglichen Bedarfs („Nerola
   Classic“). Die Stämme sind erfundene, sprachneutrale Wörter; eine Liste echter
   Produkt- und Markennamen (`ausgeschlossen`) sichert sie ab und gilt auch für den
   Spieler.
3. **Hausmarken:** Eine Firma nimmt für ein weiteres Produkt desselben Stils mit 60 %
   einen ihrer Stämme wieder; Stämme anderer Firmen nimmt sie erst, wenn keine freien
   mehr übrig sind. Derselbe Name für dasselbe Produkt ist nie zweimal vergeben.
4. **Der Spieler** benennt selbst (Markt → Produkt → „Dein Produktname“, drei
   Vorschläge); ohne Namen verkauft er unter dem Gattungsnamen, ein Hinweis erinnert
   daran.
5. ✅ **Nachfolgemodelle** (B1, `docs/FORMELN.md`): umgesetzt wie vorgeschlagen; der Text
   darunter beschreibt den alten Stand. (Neuer Name mit jeder Entwicklungsstufe, M37) gibt es noch
   nicht. Vorschlag für später: Die KI bringt mit einer neuen Stufe ein Nachfolgemodell
   unter demselben Stamm heraus („Kelvor M80“ → „Kelvor M90“).

## N Manager-System (Vorgabe `docs/MANAGER.md`, Auftrag vom 06.10.2026)

✅ **MA0 umgesetzt** (Einzelheiten: `docs/FORMELN.md` und `docs/FORTSCHRITT.md`, MA0).
🟡 Vorläufig entschieden, abweichend von der Vorgabe oder dort offen:

1. **Vorlegen statt Vorschlagsliste:** Die Regeln liefern ihre Optionen nicht vorab als
   fertige Befehlslisten zum Ausführen, sondern legen ihr Vorhaben einem Entscheider vor
   und handeln danach selbst wie bisher. Grund: Manche Regeln planen einen Schritt erst
   nach dem vorigen (Grundstück kaufen oder pachten erst nach dem Kredit, Bauzahl nach der
   Grundstücksgröße). Eine fertige Liste hätte die KI in Einzelfällen verändert; so bleibt
   sie sicher bitgleich. Wählt ein Manager eine andere Option, wird sie als Befehlsliste
   ausgeführt.
2. **Bewertung auf Nachfrage:** Betrag und Wirkung rechnet erst, wer sie braucht (Manager,
   Ansichten); die KI bildet die Entscheidungen gar nicht erst. 1 000 KI-Firmen werden
   dadurch nicht langsamer (MA6).
3. **Ohne Schätzung** bleiben Einkauf, Preisuntergrenze, Lieferungen, Namen, Forschung,
   Weiterentwicklung und Kaufangebote; den Mehrabsatz durch Werbung schätzt erst MA2.
4. **Später:** Die Antworten der KI auf Kaufangebote und ihre Gebote bei Versteigerungen
   werden mit der Landes- und Kontinentebene (MA3) zu Entscheidungen.
5. **Kredite** zählen mit ihrem Betrag auf das Budget (Vorgabe 5.1 nennt sie nur als
   Befugnis von Finanzressort und CEO).

✅ **MA1 umgesetzt** (Stellen der Standorte, Managermarkt, Gehälter, Routine).
🟡 Vorläufig entschieden:

6. **Markt über Kontinente hinweg:** Jede Stelle lässt sich aus jedem Kontinent besetzen;
   der Markt zeigt den eigenen Kontinent zuerst. Die Gehaltsforderung richtet sich nach
   dem Land des Standorts, nicht nach der Heimat.
7. **Stellen ohne Aufgaben** (Logistik, Finanzen) sind im Organigramm sichtbar, aber erst
   mit ihren Themen besetzbar (Logistik Stufe 2, Finanzen MA5). Der Befehl selbst erlaubt
   sie schon. Seit MA2 haben Personal (Lohnaufschlag) und die Laborleitungen (Forschungs-
   und Entwicklungsziel) Aufgaben.
8. **Verkauf eines Standorts:** Die Stellen enden ohne Abfindung; die Manager gehen
   nicht mit an den Käufer.
9. **Namen:** Die Namensgruppen decken Europa, Amerika, Japan und China ab; Manager aus
   Afrika, Arabien, Süd- und Südostasien tragen die Namen der Standardgruppe (englisch).
   Vorschlag: Gruppen arabisch, indisch, ostafrikanisch, portugiesisch-brasilianisch,
   koreanisch und türkisch in `data/ki/namen.yaml` ergänzen (betrifft auch neue
   KI-Firmen).

✅ **MA2 umgesetzt** (Budget je Stelle, Anliegen, Postfach, Runden-Halt, Bündelung,
Rückmeldung; Einzelheiten: `docs/FORMELN.md`, MA2).
🟡 Vorläufig entschieden:

10. **Einkauf im Budget:** Ein neuer Einkaufsauftrag zählt nur mit dem Teil, um den sein
    Höchstpreis über dem üblichen Höchstpreis der Regeln liegt (Marktpreis plus Aufschlag)
    – sonst würde jeder Routineeinkauf das Budget sofort aufbrauchen.
11. **Risikoneigung und Fragefreude** wirken noch nicht: Die Stelle empfiehlt nach
    Urteilsvermögen und Fachkompetenz und fragt genau dann, wenn Budget oder Befugnis
    nicht reichen. Vorschlag: Mit MA3 verschiebt die Risikoneigung die Wahl zwischen
    Optionen mit breiter Prognose, die Fragefreude lässt knapp unter dem Budget fragen.
12. **Struktur am Quartalsende:** Stilllegen, Verkaufen, Wiederanfahren und Ausbau prüfen
    die Stellen nur am letzten Tag eines Quartals (wie die Überkapazitätsprüfung der KI),
    die Routine an jedem Prüftermin.
13. **Erledigt durch eigene Entscheidung:** Ein Befehl des Spielers (oder eine spätere
    Entscheidung der Stelle im Budget) schließt offene Anliegen, deren Optionen dieselbe
    Anlage, dasselbe Produkt im Verkauf oder Einkauf, dieselbe Lieferung, den Lohn, den
    Ausbau, das Labor oder die Lagerstätte desselben Standorts betreffen.
14. **Runden-Halt:** Vorgabe „bei wichtigen“; wichtig ist, was nicht zur Routine gehört
    (Überkapazität, stillgelegte Anlagen, Wiederanfahren, Ausbau, Forschung,
    Weiterentwicklung). Die Einstellung merkt sich der Browser bzw. die App, nicht der
    Spielstand. Die Meldung eines neuen Anliegens hält „bis zur nächsten Meldung“ nur noch
    nach dieser Einstellung an, nicht mehr als Warnung.
15. **Bündelung:** gleiches Thema und gleiche Art der Empfehlung; „Empfehlung für alle
    übernehmen“ antwortet jedem Anliegen der Gruppe mit „Entscheide selbst“.
16. **Kredit als Grund:** Optionen mit Kredit legt jede Stelle vor (Finanzen und CEO
    entscheiden ab MA5), auch wenn der Betrag ins Budget passte.
17. **Zurücknehmen:** „Nicht mehr fragen“ und „Ablehnen“ hebt „Wieder fragen“ in der
    Organisation auf (Befehl `AskAgain`; in der Vorgabe nicht vorgesehen).

✅ **MA3 umgesetzt** (Land und Kontinent, Weiterleitung, Budget-Vorgaben, Deckel,
strategische Anliegen; Einzelheiten: `docs/FORMELN.md`, MA3).
🟡 Vorläufig entschieden:

18. **Keine Routine von oben:** Land und Kontinent übernehmen nicht die wöchentliche
    Routine von Standorten ohne Stelle (Auslastung, Preise, Einkauf), nur deren Struktur
    am Quartalsende, Kraftwerke, Lagerstätten, neue Standorte und die Werbung. Vorschlag:
    so lassen – sonst ersetzt eine Landesleitung alle Werksleitungen.
19. **Kaufangebote und Versteigerungen** bleiben beim Spieler; sie kommen mit dem Vorstand
    (MA5) zu Entscheidungen (Abweichung von MA0, Punkt 4).
20. **Markteintritt** in Länder ohne eigenen Standort gibt es bei den Regeln der KI nur
    über neue Ketten (`diversify`); er kommt mit dem CEO (MA5).
21. **Einmal je Tag:** Ausbau, Kraftwerk und Lagerstätte entscheiden die Stellen einer Firma
    höchstens einmal je Land und Produkt am Tag – sonst bauten zwei Werke desselben Landes
    am Quartalsende doppelt.
22. **Bündeln nur am selben Tag:** Strategische Anliegen entstehen aus Anliegen desselben
    Tages (Quartalsende); einzelne Anliegen anderer Tage bleiben einzeln (die Oberfläche
    gruppiert sie weiter).
23. **Sitzland eines Kontinents:** Land des Firmensitzes, sonst das mit den meisten
    eigenen Standorten.
24. **Gehälter höherer Stellen** sind Gemeinkosten der Firma (Kostenstelle ohne Standort);
    das Controlling je Ebene folgt mit den Zentralabteilungen (ZA).
25. **Wer zuerst bemerkt:** Eine Entscheidung beginnt bei der ersten Stelle der Kette, die
    das Thema heute bemerkt hat; hat die Landesstelle nicht hingesehen, kann der Kontinent
    für das Land entscheiden.

✅ **MA4 umgesetzt** (Strategievorgaben je Firma, Kontinent, Land und Standort mit
Herkunft; Einzelheiten: `docs/FORMELN.md`, MA4).
🟡 Vorläufig entschieden:

26. **Geltungsbereiche ohne Produkt:** Vorgaben gelten je Einheit, nicht je Produkt. Ein
    abweichender Preis für ein einzelnes Produkt geht über den Standort (oder einen festen
    Preis). Vorschlag: Produkt als zusätzlicher Geltungsbereich, wenn sich im Spiel zeigt,
    dass er fehlt.
27. **Wie gut umgesetzt wird:** Die Fähigkeiten wirken über das Bemerken am Prüftermin
    (MA1) und die Empfehlung (MA2); einen zusätzlichen Umsetzungsfehler gibt es nicht
    (er würde Preise zufällig schwanken lassen).
28. **Direkte Entscheidung hat Vorrang:** Sie gilt sofort; feste Preise bleiben dauerhaft,
    alles andere passt die Stelle an ihrer nächsten Prüfung wieder der Vorgabe an. Wer
    dauerhaft abweichen will, setzt eine Vorgabe für den Standort.
29. **Investition** umfasst Bauen, Erschließen, Standort gründen, Grundstück kaufen und
    Wiederanfahren; alle Budgets der Ebenen über dem Ort gelten zugleich; was der Spieler
    freigibt, zählt nicht. Budget als fester Betrag je Kalenderjahr (nicht als
    Umsatzanteil wie die Stellenbudgets).
30. **Liquiditätsreserve** in Monaten laufender Kosten (wächst mit der Firma) und nur für
    Investitionen; Kredite bleiben bis MA5 beim Spieler.
31. ✅ **Verkaufswege** (Lastenheft §9.2, im Kern seit M8) stehen jetzt in der
    Strategieansicht – mit ihren eigenen Geltungsbereichen Firma, Land, Produkt und
    Produkt im Land (nicht nach Standort und Kontinent, wie in M8 festgelegt).
32. **Vorschläge der Manager zu Vorgaben** (MANAGER.md §7) kommen mit der
    Strategierücksprache des CEO (MA5).
33. **Qualität, Partner, Logistik, Marketing, Forschung als Strategiefelder** folgen mit den
    Meilensteinen, die diese Entscheidungen den Managern geben (Logistik Stufe 2, Werbung
    und Forschung mit MA5/ZA).

✅ **MA5 umgesetzt** (Vorstand mit CEO und Ressorts, Kasse und Kaufangebote,
Strategieauftrag, Strategierücksprache; Einzelheiten: `docs/FORMELN.md`, MA5).
🟡 Vorläufig entschieden:

34. **Regelthemen:** Kredite und Werbung folgen der Regel, weil ihre Bewertung nur Kosten
    zählt (Zins, Mehrbudget). Vorher hätte eine urteilsstarke Stelle nie einen Kredit
    aufgenommen und – schon seit MA3 – nie die Werbung erhöht. Anliegen dazu bleiben
    wichtig. Vorschlag: den Mehrabsatz durch Werbung später schätzen (MA0, Punkt 3).
35. **Ein offenes Anliegen je Thema am Vorstand:** Kommen mehrere Kaufangebote zugleich und
    reicht das Budget nicht, ist nur eines als Anliegen offen; die übrigen beantwortet der
    Vorstand am nächsten Prüftermin (Angebote gelten zwei Monate).
36. **Sperren gelten für die Stellen,** nicht für den Spieler; Lizenzen sind nie gesperrt.
37. ✅ (B1: Warnung im Managermarkt und beim Abwerben) **CEO und kleine Firmen:** Ein CEO kostet das 15-Fache eines Akademikerlohns; eine
    Werkstatt mit 100.000 USD Startkapital geht daran in wenigen Monaten pleite. Die
    Gehaltsforderung steht im Managermarkt. Vorschlag: dort warnen, wenn das Jahresgehalt
    die Kasse übersteigt.
38. **Aggressivität der Spielerfirma:** Leitlinie „Ertrag“ = 0,5, genau der bisherige Wert;
    ohne Auftrag ändert sich nichts.
39. **Rücksprache mit dem CEO allein:** Ressorts sind dafür nicht nötig; ohne CEO gibt es
    keine Rücksprache (die Ansicht sagt, wie es dazu kommt).
40. ✅ **Stellen je Firma:** Die Suche nach dem Inhaber ist seit MA6 firmenbezogen (Land,
    Kontinent und Vorstand gibt es in jeder Firma).
41. **Umsatz je Produkt** wird je Monat aufgezeichnet (für den Bericht). Dadurch ändert
    sich der Zustands-Hash der Weltläufe, nicht ihr Verlauf: Die Protokolle bleiben gleich.

✅ **MA6 umgesetzt** (Erfahrung, Zufriedenheit, Kündigung, Leitung stellt ein, KI-Firmen
stellen ein und werben ab; Einzelheiten: `docs/FORMELN.md`, MA6). 🟡 Vorläufig
entschieden:

42. **Gehalt anpassen** ist neu: Weil die Löhne über die Jahrzehnte steigen, fiele ein
    festes Gehalt hinter den Marktwert zurück, und jeder Manager würde irgendwann
    unzufrieden. Gehälter lassen sich nur erhöhen; KI-Firmen heben unzufriedene Manager auf
    den Marktwert.
43. **Sperrfrist für Angebote** (`sperre_monate`, 12): Ohne sie würde eine KI-Firma, deren
    Angebot gehalten wurde, denselben Manager jeden Monat wieder umwerben – sein Gehalt
    stiege jedes Mal um den Aufschlag.
44. **KI-Firmen antworten sofort,** der Spieler bis zur Frist der Anliegen. Unbeantwortet,
    abgelehnt oder bei ruhendem Thema verfällt das Angebot; der Manager bleibt, die
    Zufriedenheit sinkt um `ignoriert_abzug`.
45. ✅ (B2: Schätzfehler bei Kosten und Marge nach Kompetenz, `docs/FORMELN.md`) **Manager der KI wirken nur über die Kompetenz** (Häufigkeit der Betriebsprüfung,
    Vorausplanung der Forschung); sie entscheiden nicht über Stellen-Ketten und stellen
    keine Anliegen. Vorschlag für später: die Kompetenz auch in Preis- und
    Ausbauentscheidungen wirken lassen.
46. ✅ (B1: Reiter „Wettbewerb“ → Firma → „Führung“) **Abwerben durch den Spieler:** Der Befehl `PoachManager` steht allen Firmen offen,
    die Oberfläche bietet ihn noch nicht an (es gibt keine Liste fremder Manager).
    Vorschlag: im Reiter „Wettbewerb“ die Leitungen der Firmen zeigen und dort „Abwerben“
    anbieten.
47. **Obergrenze der Erfahrung** aus der höchsten Fachkompetenz bei der Ziehung (statt
    der des Schwerpunkts): einheitlich für neue Bewerber und ältere Spielstände.

## O Hauptsitz, Zentralabteilungen, Start-ups (Vorgabe `docs/BETEILIGUNGEN.md`)

✅ **ZA1 umgesetzt** (Hauptsitz verlegen), ✅ **ZA2 umgesetzt** (Zentralabteilungen,
Vorgabe „Beteiligungen“), ✅ **ZA3 umgesetzt** (Empfehlungen, Trefferquote), ✅ **SU1
umgesetzt** (Start-ups ohne Beteiligung des Spielers), ✅ **SU2 umgesetzt**
(Beteiligungen der Firmen), ✅ **SU3 umgesetzt** (Ausgründungen, KI-Firmen beteiligen
sich), ✅ **ZA4 umgesetzt** („Paket A“ vom 07.10.2026: Punkte 5, 11, 14, 29 und 36;
Regeln: `docs/FORMELN.md`, ZA4). 🟡 Vorläufig entschieden:

1. ✅ (W2: Städte mit Akademikern und Büromiete) **Land statt Stadt:** Der Hauptsitz ist
   ein Land. Städte gibt es im Spiel nicht (nur Grundstücke mit Lage Stadt, Hafen, Land);
   Vorschlag: eine Stadt erst, wenn sie etwas bewirkt (z. B. mit der Logistik).
2. **Immer sichtbar:** Die Karte „Hauptsitz und Zentrale“ steht von Anfang an in der
   Organisation – sonst fände der Spieler nicht, wo er die erste Abteilung einrichtet.
3. **Kosten** 250.000 USD plus 25.000 USD je Angestelltem der Zentrale, Dauer sechs
   Monate, 60 % ziehen mit (Schätzungen in `parameter/zentrale.yaml`).
4. **Kriegsrisiken** des Sitzlands kommen mit Stufe 4.
5. ✅ **KI-Firmen verlegen ihren Sitz** (ZA4): zum Jahresbeginn in ein Land, in dem sie
   mindestens ein Viertel ihres Umsatzes machen und das nicht viel ärmer ist, wenn die
   Ersparnis an Gewinnsteuer und Löhnen der Zentrale die Kosten in drei Jahren deckt;
   danach zehn Jahre Ruhe (Punkt 39).
6. ✅ (W2: Grenze durch die Akademiker der Stadt) **Angestellte ohne Einzelpersonen,
   sofort:** Die Zahl der Angestellten je Abteilung gilt ab sofort, ohne
   Einstellungskosten, Abfindung oder Grenze durch den Akademiker-Pool des Landes. Vorschlag: den Pool erst prüfen, wenn Zentralen groß werden (Stufe 2).
7. **Kosten je Angestelltem:** Lohn der Gruppe `akademiker.kaufmaennisch` im Sitzland plus
   15.000 USD Büro im Jahr – 1900 in Deutschland rund 4.000 USD im Monat. Mit der
   Ressortleitung kostet eine kleine Abteilung rund eine halbe Million USD im Jahr; eine
   Werkstatt mit je einem Angestellten in Finanzen und Marketing ist nach einem Jahr
   zahlungsunfähig (Szenariotest).
8. **Lizenzen sind Sache des Rechts:** Das Thema `lizenz` (kaufen und verkaufen) gehört dem
   neuen Ressort Recht, Übernahmen und Antworten auf Angebote für Standorte und Bereiche
   dem Ressort Strategie; der Vorstand bietet je Prüfung das beste Übernahmeziel und die
   beste Lizenz an (vorher eines von beiden), mit arbeitender Abteilung bis zu ⌊K⌋ je Art.
9. **Beteiligungsbudget:** Es zählen alle Käufe des Jahres – auch die, die der Spieler
   selbst macht, und Zuschläge in Versteigerungen – und die offenen Gebote. Die
   **Freigabegrenze** gilt für die Vorstandsfachstelle des Ressorts; darüber entscheidet
   der CEO in seinem Budget, sonst fragt die Stelle den Spieler.
10. **Wirkungen** (Schätzwerte in `zentrale.yaml`): Finanzen bis 40 % weniger
    Risikoaufschlag, Personal bis 50 % mehr Erfahrung, Marketing bis 30 % stärkere Werbung;
    Strategie und Recht wirken über die Zahl der Länder und Technologien, die sie prüfen.
11. ✅ **KI-Firmen richten Zentralabteilungen ein** (ZA4): nach ihrem Umsatz, mit
    denselben Befehlen und Wirkungen wie der Spieler (Punkte 37 und 38).
12. **Welt mit neuen Ressorts:** Die zwei neuen Bereiche ändern die Ziehung der Manager
    (mehr Fähigkeiten je Manager); deshalb weicht ein Weltlauf mit gleichem Startwert vom
    Stand vor ZA2 ab. Mit den alten Daten ist er bitgleich.
13. **Wer schätzt:** Jede Übernahme und Lizenz, die der Vorstand anbietet, schätzt die
    Stelle des Ressorts (sonst der CEO) – auch ohne Abteilung; die Abteilung macht die
    Schätzung genauer. Bewertet wird jede Schätzung, auch wenn das Angebot nicht abgeht.
14. ✅ **Treffer nach dem Erfolg** (ZA4): Übernahmen zählen nach der Frist mit dem
    Grundwert, den der Gegenstand dann hat; Start-up-Empfehlungen werden ebenfalls
    bewertet; Lizenzen bleiben bei der Bewertung am Tag der Schätzung (Punkt 41).
15. **Gehaltsrunde** ist ein Regelthema: Die Personalstelle empfiehlt immer die Erhöhung
    auf den Marktwert; Umschuldung wird nach ersparten Zinsen und Gebühr bewertet.
16. **Verbesserung = Stufe der Weiterentwicklung:** Ein Start-up „Verbesserung“ zielt auf
    die nächste Stufe eines Produkts (M37), die noch niemand erreicht hat; Erfolg setzt
    die Stufe weltweit, nach der Gemeingut-Frist kennt sie jeder. Nur Produkte, die
    heute jemand herstellt (sonst arbeiteten Start-ups 1910 an Lithium).
17. **Überholt:** Wird die Technologie vorher erfunden (Geschichte oder Forschung einer
    Firma) oder die Stufe von einer Firma erreicht, geht das Start-up ein. Ohne diese
    Regel endeten viele „Erfolge“ nach dem historischen Jahr ohne jede Wirkung.
18. **Gründungsland** gewichtet mit Einwohnern · BIP je Kopf · Entwicklungsstand: Sonst
    lägen um 1900 China und Indien vorn (große Wirtschaft, kaum Forschung).
19. **Phasen und Chancen** (Schätzungen in `parameter/startups.yaml`): 12/18/24 Monate,
    Chancen 0,65/0,8/0,9, Kapital 150.000/800.000/4 Mio. USD bei 50.000 USD BIP je Kopf;
    Weltlauf bis 1940: 70 % gescheitert, 15 Technologien 1–10 Jahre früher erfunden.
20. **Häufigkeit** wählbar beim neuen Spiel (keine, wenige 6, normal 12, viele 24 im
    Jahr); alte Spielstände bekommen „normal“.
21. **Investoren außerhalb des Spiels** (`Holder::Investors`) finanzieren die Runden
    ganz; Spieler, Firmen und Fördergeld seit SU2, KI-Investoren mit SU3.
22. **Beendete bleiben zehn Jahre** in der Liste, historische Erfinder immer (so gründet
    jeder nur einmal). Ein Spielstand wächst dadurch kaum (rund 150 Einträge).
23. **Firmen sind die Eigner:** Der Spieler beteiligt sich mit seiner Firma; die
    Beteiligung steht in ihrer Bilanz (neues Konto Finanzanlagen) wie bei KI-Firmen.
24. **Zusage statt Sofortkauf:** In einer offenen Runde geht das Geld sofort in die
    Finanzanlagen, Anteile gibt es erst, wenn die Runde schließt (sofort, wenn die Zusagen
    sie decken). Kommt sie nicht zustande, fließt die Zusage zurück.
25. **Wert bei Erfolg** = Bewertung nach der letzten Runde · 1,3 (Schätzung): Ein Dollar
    der ersten Runde bringt im Mittel rund das 2,8-Fache, der zweiten das 1,9-Fache, der
    dritten das 1,2-Fache; die meisten Einsätze sind verloren.
26. **Börsengang statt Börse:** Ohne Mehrheit endet ein Erfolg mit einer Auszahlung zum
    Wert bei Erfolg an alle Eigner; eine Börse mit Kursen gibt es nicht (Vorschlag: mit
    Stufe 3). Spielen KI-Firmen mit, wird daraus eine neue KI-Firma im Land des Start-ups
    mit der Technologie – aber nur, wenn ein Produkt der Technologie dort Absatz findet;
    sonst bleibt nur die Erfindung in der Welt.
27. **Tochterfirmen finanziert die Mutter** am nächsten Monatsanfang ganz, solange ihre
    Kasse reicht (zählt zum Budget „Beteiligungen“); sonst kommen Investoren dazu und die
    Mutter wird verwässert – unter der Mehrheit ist es keine Tochter mehr.
28. **Fördergeld wirkt nur auf die laufende Phase** und höchstens bis `chance_max`
    (0,98); es zählt nicht zum Budget „Beteiligungen“, weil es Forschungsaufwand ist.
29. ✅ **Verkauf an Firmen** (ZA4): Neben dem Sofortverkauf an Investoren (20 %
    Abschlag) bietet der Spieler seinen ganzen Anteil allen Firmen an; zum nächsten
    Monatsanfang kauft das beste Gebot ab seinem Mindestpreis (Punkt 40).
30. **Meldungen** bekommt der Spieler zu seinen Beteiligungen (Rückzahlung, Verlust,
    Forschungsbonus, Tochter, Auszahlung, Übernahme, Börsengang) und wenn aus einem
    Start-up eine neue Firma wird; was KI-Firmen untereinander tun, meldet das Spiel nicht.
31. **Forschungsbonus** beim Scheitern: 20 % des heutigen Forschungsaufwands für die
    Mehrheit – ein Trost, der das Risiko einer großen Beteiligung etwas senkt.
32. **KI-Investoren** sind die Investoren außerhalb des Spiels (seit SU1): Sie decken
    Runden und kaufen verkaufte Anteile. „KI-Firmen bieten mit“ heißt: KI-Firmen sagen in
    Runden zu, kaufen Anteile und gliedern ein – mit denselben Befehlen und Prüfungen wie der
    Spieler (SU3).
33. **Ausgründen** geht nur für Ziele, die ein Start-up haben kann (noch nicht erfundene
    Technologie, noch von niemandem erreichte Stufe), und erst ab 10 % Fortschritt: Sonst
    ließe sich mit einem leeren Labor und einem Sofortverkauf Geld drucken. Die Phase folgt
    dem Fortschritt (ab einem Drittel Prototyp, ab zwei Dritteln Marktreife). KI-Firmen
    gründen nur aus, was vor der Geschichte fertig werden kann und woran kein anderes
    Forschungszentrum arbeitet: Ihre Labore forschen höchstens zwei Jahre voraus und
    entwickeln meist Produkte, an denen auch andere arbeiten – im ersten Weltlauf wurden
    solche Ausgründungen fast alle überholt. Dem Spieler zeigt die Ansicht Vorlauf,
    Restdauer und Konkurrenz und warnt, verbietet aber nichts.
34. **Tochterfirmen zahlen pro rata:** Seit SU3 sagt die Mutter in jeder Runde nur ihren
    Anteil zu, Investoren den Rest; so bleibt eine Ausgründung mit 60 % eine Tochter mit
    60 % und das Risiko ist geteilt. Eingegliederte Töchter (100 %) zahlt die Mutter weiter
    ganz.
35. **Wegkaufen** prüft eine KI-Firma nur für Start-ups, deren Ziel ihr nützt (Technologie
    für ihre Branchen, Stufe eines ihrer Produkte), an denen sie schon mindestens 10 % hält,
    mit Chance ≥ 30 % und für höchstens 20 % ihrer Kasse; eine Sperrminorität anderer (auch
    des Spielers) verhindert es. Ohne die Bedingung „schon beteiligt“ übernahmen im
    Weltlauf die KI-Firmen fast jedes nützliche Start-up, bevor es eine Runde weiter war.
36. ✅ **Ausfallquote im Richtwert** (ZA4): Im Weltlauf 1900–1940 (100 KI-Firmen,
    „normal“) scheitern 64 % der in den letzten zehn Jahren beendeten Start-ups
    (Richtwert 60–70 %). Die 77–83 % der Läufe bis 1915 kamen vom Anlauf: Fehlschläge
    enden früher als Erfolge, die alle drei Phasen brauchen (mindestens 54 Monate). Die
    Chancen der Phasen bleiben.
37. **Zentrale der KI nach Umsatz** (ZA4): Zum Jahresbeginn darf die Zentrale einer
    KI-Firma 0–2 % ihres Umsatzes der letzten zwölf Monate kosten (nach Kompetenz), mit
    Verlust nichts. Die Abteilungen kommen in der Reihenfolge Finanzen (nur mit Kredit),
    Marketing (nur mit Werbung), Personal (ab zehn Managern), Strategie, Recht – jede voll
    besetzt. Ein Nutzen in Geld lässt sich nur für Finanzen und Marketing schätzen; der
    Anteil am Umsatz begrenzt, was eine Firma für alle ausgibt.
38. **Was die Zentrale der KI tut:** dieselben Wirkungen wie beim Spieler (Kredite,
    Schulung, Werbung) und dazu: Finanzen schulden um, Strategie sucht Übernahmen in den
    beobachteten Ländern und sieht Start-ups schärfer, Recht prüft Lizenzen. Empfehlungen
    und Trefferquoten gibt es bei KI-Firmen weiter nicht (ihre Leitungen werden nicht
    bewertet). Leitungen stellt die KI wie den CEO ein (nach dem CEO, vor den Standorten).
39. **Kein Umzug in ein viel ärmeres Land** (`bip_anteil_min` 0,75): Ohne diese Bedingung
    zog im ersten Testlauf eine deutsche Weberei 1902 nach Indonesien, um Löhne für einige
    Angestellte der Zentrale zu sparen. Mit ihr verlegen KI-Firmen ihren Sitz vor allem
    wegen der Gewinnsteuer und selten (Weltlauf: siehe `docs/FORTSCHRITT.md`, ZA4).
40. **Bieterverfahren statt Verhandlung:** Der Spieler nennt einen Mindestpreis, die
    KI-Firmen bieten bis zum nächsten Monatsanfang, das höchste Gebot kauft zu seinem
    Preis; der Spieler erfährt sonst das beste Gebot. Gebote: Wert des Anteils mit dem
    Gebotsaufschlag der Kaufangebote, höchstens der erwartete Wert über dem
    Mindestertrag. Eine Sperrminorität eines Dritten verhindert, dass ein Käufer so über
    die Mehrheit kommt („Verkauf an Konkurrenten blockieren“). Noch nicht: Kaufangebote
    des Spielers für Anteile anderer Firmen und Verkäufe der KI untereinander (Vorschlag:
    mit der Börse, Stufe 3).
41. **Treffer bei Übernahmen und Start-ups** (ZA4): Übernahme: Angebot ≤ Wert nach den
    Regeln · Grundwert dann / Grundwert am Tag der Schätzung (der Markenwert eines Bereichs
    zählt wie am Tag der Schätzung). Start-up-Empfehlung: Erfolg ist ein Treffer,
    Scheitern keiner, sonst zählt der Wert des Anteils mit der wahren Chance gegen den
    Einsatz. Je Start-up und Leitung eine ausstehende Bewertung (sonst zählte jede
    monatliche Empfehlung derselben Runde).

## P Paket B und die Stufen 2–5 (Auftrag vom 07.10.2026: „alles außer Stufe 6“)

✅ **B1 umgesetzt** (Gehalt über der Kasse, Abwerben, Nachfolgemodelle; Regeln:
`docs/FORMELN.md`, Abschnitt B1). 🟡 Eigenständig entschieden:

1. **Nur eine Warnung:** Ein Gehalt über der Kasse wird angezeigt, nicht verboten – eine
   Firma mit Kredit oder sicheren Einnahmen darf teuer einstellen.
2. **Alle Manager einer Firma** stehen unter „Führung“, nicht nur die Leitungen: Auch eine
   gute Fachkraft lässt sich abwerben. Die Fähigkeiten zeigt die Einschätzung des Spielers
   wie im Managermarkt.
3. **Abwerben nur auf freie Stellen:** Das Angebot nennt eine freie Stelle des Spielers
   (wie der Befehl seit MA6); eine besetzte Stelle müsste man erst räumen.
4. **Nachfolgemodelle nur für Modellnamen** (Stil `technik`); Markennamen für Waren des
   täglichen Bedarfs bleiben. Die KI benennt mit jeder Stufe um, der Spieler bekommt
   einen Vorschlag. Mit den Zahlen der Daten springt „M80“ auf „M100“ (90 steht nicht in
   der Liste); nach der größten Zahl folgen „II“, „III“ … bis „X“.

✅ **W1 umgesetzt** (Schulung je Standort; Regeln: `docs/FORMELN.md`, Abschnitt W1).
🟡 Eigenständig entschieden:

1. **Ein Niveau je Standort**, nicht je Arbeitskräftegruppe: Die Wirkung (weniger Arbeit,
   bessere Qualität) gilt für alle Gruppen des Standorts. Getrennte Niveaus je
   Qualifikation wären genauer, aber in der Oberfläche kaum zu steuern.
2. **Ziel statt Budget:** Der Spieler setzt ein Zielniveau; die Kosten folgen dem Ziel
   (Anteil der Lohnsumme), die Wirkung dem erreichten Niveau. Ohne Ziel sinkt das Niveau
   langsam (Fluktuation).
3. **Strategie als Rückfall:** Ohne eigenes Ziel gilt die Strategievorgabe „Schulung“
   (Standort, Land, Kontinent, Firma) – so hält die Personalstelle ein Niveau auf jeder
   Ebene, wie im Lastenheft §5.4 verlangt.
4. **Forschungszentren schulen nicht** (Forscher sind Akademiker; die Forschung hat eigene
   Regeln).
5. **KI nach Kompetenz:** KI-Firmen schulen bis 0 % (Kompetenz 0) bis 80 % (Kompetenz 1).
   Ihre Ausbauschätzung rechnet die Schulung nicht ein – sie unterschätzt den Vorteil
   leicht, wie ein vorsichtiger Planer.
6. **Weiterbildung der Manager** gab es schon: Die Personalabteilung (ZA2) hebt die Chance
   der Manager auf Erfahrung um bis zu 50 %. W1 fügt dafür nichts Neues hinzu.

✅ **C3 umgesetzt** (neue Märkte; Regeln: `docs/FORMELN.md`, Abschnitt C3). 🟡 Eigenständig
entschieden:

1. **Eigener Weg für Neulinge:** Teure oder knappe Märkte mit 1 bis unter
   `einstieg_firmen_max` Herstellern bekommen je Quartal einen weiteren (bis sechs Märkte
   je Quartal bei 100 KI-Firmen), statt in der allgemeinen Rangliste gegen Stahl und Öl zu
   verlieren. Es baut die reichste Firma, die ein Verfahren kennt.
2. **Offen:** Akkus bleiben bis 2020 knapp, weil Kobalt (Bergbau mit Arbeitskräftemangel)
   in beiden Läufen das Vierfache seines Richtpreises kostet; mit mehr Geräteherstellern
   wird es spürbarer. Vorschlag: Kobalt-Lagerstätten und Arbeitskräfte im Kongo prüfen
   (Balancing, Stufe 6).

✅ **C4 umgesetzt** (Plausibilität 1900–1930; Regeln: `docs/FORMELN.md`, Abschnitt C4).
🟡 Eigenständig entschieden:

1. **Historische Firmen im Marktmaßstab:** Ihre Anlagen bekommen die passende Größe statt
   mindestens einer mittelgroßen – das war der Hauptgrund für die Überkapazität bei Erz,
   Kohle und Stahl.
2. **Ausbau nur in ausgelasteten Märkten:** Liegt die geplante Auslastung aller Anlagen
   eines Produkts weltweit unter 85 % (wie `auslastung_normal`), baut niemand aus. Neue
   Hersteller über Lücken (Diversifizierung, C3) bleiben möglich.
3. **Stahl braucht mehr Arbeit:** Bessemer und Siemens-Martin 1,5-mal so viele Stunden
   (USA 1900 rund 58 Stunden je Tonne einschließlich Walzen); die Marge zum Richtpreis
   1900 sinkt von 44 auf 39 %.
4. **Auto-Nachfrage:** Kaufschwelle 16 statt 80. Gilt für alle Epochen; nach 1990 steigt
   damit die Nachfrage in ärmeren Ländern.
5. **Startwerkstatt:** Nägel 1.800 statt 1.900 USD/t, mehr Arbeit je Tonne. Im ersten Jahr
   verdient die Werkstatt rund 40 % ihres Startkapitals.

✅ **W2 umgesetzt** (Zentrale in der Stadt; Regeln: `docs/FORMELN.md`, Abschnitt W2; löst
die Punkte O 1 und O 6). 🟡 Eigenständig entschieden:

1. **Städte aus Natural Earth:** bis fünf je Land (Hauptstadt und die größten Orte ab
   100.000 Einwohnern), deutsche Namen über eine Liste im Skript. Die Einwohner gelten als
   fester Anteil an der Bevölkerung des Landes – Städte wachsen mit dem Land, nicht für
   sich (Berlin 1900 so groß wie heute im Verhältnis).
2. **Akademiker der Stadt:** viermal so dicht wie im Landesschnitt, höchstens alle des
   Landes; 5 % davon stehen Zentralen offen (Schätzungen). Die Zentrale rechnet in realer
   Größe und nimmt den Werken (im Marktmaßstab) keine Arbeitskräfte weg.
3. **Anteilige Besetzung:** Wollen die Firmen einer Stadt zusammen mehr Stellen, als es
   Akademiker gibt, bekommt jede ihren Anteil – wer zuerst kam, hat keinen Vorrang. Nicht
   besetzte Stellen kosten nichts.
4. **Bürokosten** folgen dem Preisniveau des Landes und der Größe der Stadt (Elastizität
   0,15): bisher kostete ein Büro überall 15.000 USD im Jahr.
5. **Umzug im Land** kostet die Hälfte und dauert halb so lange wie ein Umzug ins Ausland.
   KI-Firmen ziehen in die Stadt mit den meisten Akademikern, wenn eine Abteilung nicht voll
   besetzt ist; ins Ausland gehen sie in die Hauptstadt.

✅ **W3 umgesetzt** (Zölle; Regeln: `docs/FORMELN.md`, Abschnitt W3; Entscheidungen bei
Punkt A 2). Zusätzlich 🟡: Ein teurer Markt (Käufer zahlen mindestens
`einstieg_preisfaktor` · Richtpreis) darf trotz der Ausbau-Bremse aus C4 wachsen – sonst
blieb Kautschuk 1990–2026 knapp, weil Synthesewerke ohne Benzin die weltweite Auslastung
drückten.

✅ **W4 umgesetzt** (Lieferverträge; Regeln: `docs/FORMELN.md`, Abschnitt W4). 🟡
Eigenständig entschieden:

1. **Preis frei Standort des Käufers:** Fracht und Zoll trägt der Verkäufer; der Käufer
   kennt seinen Einstand sofort. Die Vorschläge der Lieferanten rechnen beides ein.
2. **Nur zwischen Spieler und KI-Firmen** (Lastenheft §9.3). Verträge der KI-Firmen
   untereinander würden die Weltläufe verändern; sie folgen bei Bedarf mit dem Balancing.
3. **Die KI antwortet sofort** nach festen Regeln (Preis gegen eigenen bzw. Marktpreis,
   höchstens die Hälfte ihrer Leistung oder ihres Bedarfs, Strafe bis 50 %). Ablehnungen
   nennen den Grund. Verhandeln mit Gegenangebot gibt es (noch) nicht.
4. **Wer ist schuld an fehlender Menge:** Fehlt Ware, zahlt der Verkäufer; fehlt dem Käufer
   das Geld, zahlt er. Die Strafe ist ein Anteil am Wert der fehlenden Menge.
5. **Kündigen kostet** die Strafe auf bis zu drei Monatsmengen; ein Angebot zurückziehen
   ist kostenlos. Verträge enden ohne Strafe, wenn ein Standort den Besitzer wechselt
   oder eine Firma pleitegeht.
6. **Angebote der KI** an den Spieler: höchstens eines je Monat und Produkt des Spielers
   (Chance 25 %), dasselbe Standortpaar erst wieder nach zwölf Monaten.

✅ **W5 umgesetzt** (Logistik; Regeln: `docs/FORMELN.md`, Abschnitt W5). 🟡 Eigenständig
entschieden:

1. **Drei Wege je Firma, nicht je Ladung:** Die Firma wählt Frachtmarkt, staatlichen
   Transport oder eigene Flotte für alle eigenen Ladungen (Umlagerungen und Lieferungen aus
   Verträgen). Einkäufe am Markt liefern weiter die Händler.
2. **Flotte als Kapazität in Tonnenkilometern** je Monat statt einzelner Fahrten mit
   Fahrplänen; Fahrzeuge fahren nicht wirklich zwischen Ländern hin und her. Nutzlast und
   Geschwindigkeit gelten im laufenden Jahr (Modernisierung steckt im Unterhalt).
3. **Kaufpreise je Verkehrsmittel als Daten** (geschätzt aus zeitgenössischen Preisen,
   Kaufkraft 2026): Fuhrwerk 15.000 USD, Lastwagen 100.000–150.000, Güterzug 0,8–4 Mio.,
   Dampfschiff 7,5–13 Mio., Motorschiff 10–45 Mio., Tanker 6–90 Mio., Containerschiff
   25–80 Mio.
4. **Betriebskosten aus der Marktfracht:** Die Marktfracht deckt Betrieb, Kapital und eine
   Marge der Logistikfirmen (15 %). Eine voll genutzte Flotte des günstigsten Verkehrsmittels
   spart diese Marge; eine halb leere kostet mehr. Fahrzeuge, die teurer fahren als der Markt
   (Fuhrwerk und frühe Lastwagen, wo Bahnen fahren), bleiben stehen. Die Infrastruktur der
   Länder wird beim Vergleich nicht gesondert betrachtet.
5. **Risiko je Ladung** (zu Land 2 ‰ um 1900 bis 0,3 ‰ ab 2000, zur See 6 ‰ bis 1 ‰;
   staatlicher Transport halb so viel). Eine verlorene Vertragslieferung trägt der
   Verkäufer. Versicherungen gibt es (noch) nicht.
6. **Luftfracht** bleibt beim Frachtmarkt; Flugzeuge kauft man in Stufe 5 (P2).
7. **Die KI** kauft Fahrzeuge, die ihre Ladungen des Vormonats ganz füllen, bis die Flotte
   die Hälfte trägt (höchstens 20 % der Kasse je Monat), und fährt dann auch für andere.

## Reihenfolge der neuen Punkte

✅ Entschieden am 06.10.2026: Regionen und Grundstücke vor den restlichen Epochen – sie
ändern, wo und wie die Firmen bauen; die Epochen würden sonst zweimal abgestimmt. Die
Epochen bekommen deshalb neue Nummern.

| Meilenstein | Inhalt |
| --- | --- |
| M34 | Regionen (Abschnitt I): Länderdaten, Karte, Verweise, Spielstände |
| M35 | Grundstücke mit Lage (Abschnitt H, Punkte 1–8 und 10): Kern, Daten, KI, Oberfläche |
| M36 | Anlagen in fünf Größen (Abschnitt H, Punkt 9) |
| M37 | Weiterentwicklung erforschter Produkte (Abschnitt J) |
| M38 | Pleiten: Standorte weitergeben (Abschnitt K) |
| M39–M41 | Produkte 1965–1989, 1990–2009, 2010–2026 (Abschnitt G; Daten für 1965–1989 liegen als Entwurf vor) |
| M42 | Produktnamen je Firma (Abschnitt M) |
| MA0–MA6 | Manager-System (`docs/MANAGER.md`, Abschnitt 11; vorgezogen aus Stufe 2, nach M41) |
| ZA1–ZA3 | Hauptsitz und Zentralabteilungen (`docs/BETEILIGUNGEN.md`, Abschnitt 8) |
| SU1–SU3 | Start-ups und Beteiligungen (`docs/BETEILIGUNGEN.md`, Abschnitt 8; vorgezogen aus Stufe 3) |

✅ Entschieden am 06.10.2026 abends: Manager-System und Beteiligungen „direkt mit
einbinden“ – in dieser Reihenfolge nach M41.

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
