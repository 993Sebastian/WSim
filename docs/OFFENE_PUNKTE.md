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
