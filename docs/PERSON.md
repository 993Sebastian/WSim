# Vorgabe Lebenszyklus, Spielerfigur und Privatvermögen

Vorgabe des Auftraggebers vom 07.10.2026, abgestimmt in Claude Docs („WSim – Vorgabe
Lebenszyklus, Spielerfigur und Privatvermögen“). Alle Fragen sind entschieden
(Abschnitt 12). Meilensteine PE1–PE6 (Abschnitt 11). Gilt bei Widerspruch vor
Lastenheft §2 (Satz „personenunabhängig“), §11.1 (Startkapital), §11.2 (letzter Satz)
und §11.3 (Satz 2); Entscheidungen im Lastenheft §18.7.

## 1 Zusammenfassung

Menschen im Spiel bekommen ein Lebensalter, und der Spieler wird vom anonymen
Eigentümer zu einer Person mit eigenem Vermögen.

- **Das Spiel beginnt mit der Person, nicht mit der Firma.** Der Spieler gibt vor
  Spielbeginn ein Startgeld ein. Davon gründet er seine erste Firma selbst, wann er will.
- **Manager altern.** Sie haben ein Geburtsdatum, sammeln altersabhängig Erfahrung,
  gehen in den Ruhestand und fallen selten unerwartet aus. Die Nachfolge regelt ein
  Anliegen mit Vorlauf.
- **Die Spielerfigur wird eine Person.** Name, Geburtsjahr, Familie, Lebenslauf und
  Rollen in Firmen. Sie ist immer sterblich. Eine Partie läuft bis zu 200 Jahre, also
  übernimmt bei Tod oder Rückzug ein Erbe: Familiengeschichte über drei bis vier
  Generationen.
- **Die Person hat Privatvermögen.** Einnahmen aus Gehalt und Dividenden, Ausgaben für
  Lebensstil und Steuern. Damit kauft und verkauft sie Firmenanteile und ganze Firmen,
  gründet Firmen und gibt Kapital in ihre Firmen.

Die Börse (K1) ist nicht Teil dieser Vorgabe. Sie nutzt später dasselbe Privatkonto und
dieselben Eigentumsregeln; K3 (Investoren) baut auf PE4 und PE5 auf.

## 2 Ausgangslage im Code (Stand 07.10.2026, nach W2)

| Bereich | Heute | Lücke |
| --- | --- | --- |
| Spielerfigur | Lastenheft §2: „kein Alter, kein Lebenslauf, keine Nachfolge“ | Regel wird aufgehoben |
| Eigentum | `Company.owners` mit `Holder::Player`, `Company`, `Private`, `Investors`; `GameState.player` ist genau eine Firma | Person hat kein Konto und steuert nur eine Firma |
| Manager | Eigener Bestand im `GameState`; Fähigkeiten, Erfahrung bis `potential`, Zufriedenheit, Kündigung, Abwerbung (MA6) | Kein Geburtsdatum; ein Manager kann 1900 bis 2100 arbeiten |
| Gehälter | Gehaltsforderung und Marktwert steigen mit den Löhnen (MA1, MA6) | Spieler bezieht kein Gehalt |
| Steuern | `dividend_tax` je Land als Zeitreihe | Ungenutzt, keine Ausschüttung |
| Start-up-Anteile (SU2) | Gehalten von Firmen (`Holder::Company`) | Person kann nicht privat investieren |
| Kaufangebote (M30, M31) | Standorte, Bereiche mit Marke, Lizenzen | Keine Anteile, keine ganzen Firmen |
| Spielende | Zahlungsunfähigkeit der Spielerfirma | Mit Privatvermögen neu zu fassen |
| Startkapital | `settings.start_capital` geht in die Firma | Spiel beginnt immer mit fertiger Firma |

`docs/MANAGER.md` §9 kündigt die „Personenebene mit Vermögen, Beteiligungen, Dividende,
Abberufung eines CEO“ bereits an; diese Vorgabe füllt sie aus.

## 3 Alterung der Manager

Jeder Manager bekommt ein Geburtsdatum; das Alter wird am Spieltag berechnet, nicht
gespeichert. Alle Zahlen sind Parameter in `data/`.

Alter beim Eintritt in den Pool (Normalverteilung, abgeschnitten):

| Ebene der Fokusstelle | Spanne | Mittel |
| --- | --- | --- |
| Standort | 26–50 Jahre | 35 |
| Land | 32–55 Jahre | 42 |
| Kontinent | 38–58 Jahre | 47 |
| Vorstand und CEO | 42–62 Jahre | 52 |

Wirkung des Alters:

- **Erfahrung:** `experience_chance` (MA6) × Altersfaktor: unter 35 Jahren 1,5, 35–54
  Jahre 1,0, ab 55 Jahren 0,5. Obergrenze `potential` bleibt.
- **Risikofreude** sinkt ab 40 Jahren um 0,3 Punkte je Jahr, höchstens um 10 Punkte.
- **Abbau ab 65 (langsam):** Erkennen und Führung sinken mit 10 % Chance je Jahr um einen
  Punkt. Urteilsvermögen und Fachwissen bleiben.
- **Gehalt:** kein eigener Altersaufschlag; die Gehaltsforderung folgt den Fähigkeiten.

Anzeige: Alter als Zahl, nicht als Stufe. Alte Spielstände: Geburtsdatum beim Laden aus
dem Zufallsstrom des Managers, passend zu seiner Stelle.

## 4 Ruhestand, Tod und Nachfolge der Manager

Ruhestand:

- Neue Länder-Zeitreihe `ruhestandsalter` (mit `annaeherung: true`), Standard 65.
- Persönliche Abweichung −5 bis +5 Jahre bei der Erzeugung; Anzeige „plant Ruhestand mit
  etwa …“.
- **Vorwarnung:** zwölf Monate vorher Anliegen „Nachfolge regeln“ an die nächste besetzte
  Stelle darüber, sonst an den Eigentümer. Optionen:
    1. Nachfolger aus dem Pool suchen
    2. Einen Manager der Stelle darunter befördern
    3. Verlängerung anbieten: bis zu drei Jahre gegen Gehalt +20 %; Zusage hängt an
       Zufriedenheit und Alter
    4. Stelle nach dem Abgang unbesetzt lassen
- **Übergabe:** Tritt der Nachfolger mindestens einen Monat vorher an, laufen beide
  Gehälter parallel; der Nachfolger übernimmt offene Anliegen und Folgeberichte ohne Lücke.
- **Fristablauf:** Der Manager geht zum Termin, die Stelle ist leer, Anliegen laufen nach
  oben weiter.

Tod und Krankheit:

- Neue Länder-Zeitreihe `lebenserwartung` (Heimatland, `annaeherung: true`). Einfache
  Sterbetafel: Monatschance steigt ab 50 exponentiell und erreicht bei der
  Lebenserwartung etwa 0,5 % je Monat.
- Ein Ausfall leert die Stelle sofort; Meldung und Anliegen „Stelle neu besetzen“.

Pool und KI-Firmen:

- Freie Kandidaten altern mit und verlassen den Pool mit dem Ruhestandsalter; der Pool
  füllt sich wie heute aus `per_million_academics`.
- KI-Firmen folgen denselben Regeln; ihre Nachfolge entscheidet `AiHiringModel`.
- Ausgeschiedene Manager verlassen den Bestand, bleiben aber mit Name, Stellen und
  Trefferquote im Lebenslauf der Firma sichtbar.

## 5 Spielerfigur als Person

`Holder::Player` steht künftig für die Person.

Beim neuen Spiel:

- Name der Person, Geburtsjahr (Standard Startjahr − 30, wählbar 18 bis 60 Jahre alt),
  Wohnsitz = Startland.
- Familie: verheiratet ja/nein, Kinder 0 bis 4 (Standard: verheiratet, keine Kinder).
- **Startgeld** als frei eingegebener Betrag (Abschnitt 6).
- Die Person ist immer sterblich; keine Einstellung dafür.

Familie:

- Kinder kommen zufällig, solange die Person 22 bis 45 Jahre alt und verheiratet ist;
  höchstens vier.
- Jedes Kind hat Name und Geburtsdatum und ab 25 Jahren eine Managerkarte wie ein
  Pool-Kandidat. Es kann in jeder eigenen Firma eine Stelle übernehmen, kündigt nicht,
  wird nicht abgeworben, verlangt aber den Marktwert.

Rollen:

- **Eigentümer** jeder Firma, an der die Person Anteile hält; Kontrolle ab über 50 %
  direkt oder über eigene Firmen.
- **CEO** in höchstens einer Firma, mit Gehalt. Wer gründet, ist zunächst CEO.
- **Steuerung:** Solange in einer kontrollierten Firma kein CEO eingestellt ist, steuert
  der Spieler sie selbst. Mit CEO führt dieser die Firma; der Spieler lenkt über
  Strategieauftrag, Rücksprachen und Anliegen (MA5). Eine direkte Entscheidung in einer
  Ansicht hat weiter Vorrang (`docs/MANAGER.md` §6.3).
- Die Person hat keine Fähigkeitswerte.

Lebenslauf: Chronik aller Rollen und Ereignisse (Gründung, CEO von–bis, Käufe, Verkäufe,
Einlagen, Geburten, Erbfälle); verweist auf die Firmengeschichte (M28).

Alter, Tod und Nachfolge:

- Sterbetafel der Manager mit dem Wohnsitzland, beeinflusst vom Lebensstil (Abschnitt 8).
  Ab 70 jährlich die Meldung „Nachfolge bedenken“, falls kein Erbe bestimmt ist.
- **Erbe:** ein bestimmtes Kind, sonst das älteste; ohne Kind ein erzeugter Neffe oder eine
  Nichte (25 bis 40 Jahre). Die Partie läuft immer weiter.
- **Erbfall:** Privatkonto und Anteile gehen an den Erben abzüglich Erbschaftsteuer (neue
  Länder-Zeitreihe `erbschaftsteuer`). Reicht das Konto nicht, verkauft das Spiel
  Minderheitsanteile, zuletzt Anteile kontrollierter Firmen. Der Erbe wird neue
  Spielerfigur; seine Managerstelle wird frei.
- **Übergabe zu Lebzeiten:** jederzeit, mit derselben Steuer als Schenkung.

## 6 Spielstart und Gründung

Das Spiel beginnt mit der Person und ihrem Startgeld; eine Firma gibt es erst, wenn der
Spieler sie gründet.

Startgeld:

- Eingabe als Betrag (Zahlenfeld, Anzeige in der Landeswährung des Startlandes). Ersetzt
  `start_capital` und liegt vollständig auf dem Privatkonto.
- Vorschlag: der heutige Standard des Startkapitals. Keine Obergrenze.
- Untergrenze: Gründungssumme der günstigsten Startform im Startland und Startjahr; als
  Hinweis am Feld, darunter startet das Spiel nicht.

Gründung:

- Dialog **Firma gründen** direkt nach dem Start und jederzeit aus der Ansicht Person:
  Firmenname, Startform (Werkstatt oder Handel), Hauptsitz (Standard Startland), Einlage.
- Die Einlage muss Gebäude und Anlagen der Startform decken (FORMELN „Startformen“); der
  Rest bleibt privat. Vorschlag: 90 % des Startgelds.
- Gründungskosten je Land (Register, Notar) als Parameter: 0,5 % der Einlage, mindestens
  ein Monatslohn eines Akademikers.
- Gegründet wird, wann der Spieler will. Bis dahin laufen die Runden normal, und die Person
  kann als Investor anfangen, etwa in Start-ups (Lastenheft §17.3).
- Die geführte Einführung (M20) beginnt mit der Gründung.

Folgen im Code:

- `GameState.player` (eine Firma) entfällt. An seine Stelle treten die Person und die
  Liste ihrer kontrollierten Firmen, am Start leer.
- **Hauptfirma** ist die zuerst gegründete kontrollierte Firma, vom Spieler wechselbar.
  Etappenziele (M23), Rang (M29), beobachtete Märkte (M24) und Rundenbericht beziehen sich
  auf sie. Ohne Firma zeigt die Übersicht Privatvermögen und Rang der Person nach Vermögen.

## 7 Einkommen der Privatperson

Gehalt:

- Nur in der Firma, in der die Person CEO ist; Höhe frei, Vorschlag = Marktwert eines CEO
  mittlerer Stufe (MA1). Buchung als Personalaufwand der Stelle Vorstand.
- Mit Mitgesellschaftern höchstens das Doppelte des Marktwerts.
- **Einkommensteuer:** neue Länder-Zeitreihe `einkommensteuer` (ein Satz, Wohnsitzland);
  monatlicher Abzug, das Netto geht aufs Privatkonto.

Dividenden:

- Strategiefeld **Dividendenpolitik** je Firma (Lastenheft §5.6): Anteil am
  Jahresüberschuss nach Steuern (0–100 %, Standard 0 %) oder fester Betrag; dazu jederzeit
  eine **Sonderausschüttung**.
- Beschluss mit dem Jahresabschluss, Auszahlung im Folgemonat.
- Nur aus Gewinnrücklagen plus Jahresüberschuss; die Kasse darf nicht unter die
  Liquiditätsreserve fallen, sonst Kürzung mit Meldung.
- Verteilung nach Anteil: `Holder::Player` aufs Privatkonto, `Holder::Company` in deren
  Kasse, `Private` und `Investors` verlassen das Spiel.
- `dividend_tax` des Hauptsitzlandes wird einbehalten; zwischen eigenen Firmen mit über
  50 % entfällt sie.
- Mit CEO: Vorschlag als Anliegen zum Jahresabschluss; bei Fristablauf gilt die zuletzt
  beschlossene Politik.
- KI-Firmen schütten nach ihrem Charakter aus (z. B. 20–60 %, nichts bei Verlust oder
  knapper Kasse).

## 8 Privatvermögen

Strikt vom Firmenvermögen getrennt; haftet nicht für Firmenschulden.

- Eigenes kleines Hauptbuch in Leitwährung (Anzeige über M21). Guthaben nie negativ, keine
  privaten Kredite. Guthabenzins aus neuer Zeitreihe `sparzins` (Standard 2 %).
- Bewertung: Guthaben zum Nennwert; Firmenanteile = Anteil × Firmenwert (Bewertung aus
  M30/M31, ab K1 Börsenkurs); Start-up-Anteile = letzte Finanzierungsrunde;
  Gesellschafterdarlehen = offener Betrag. Je Anteil wird der **Einstandswert** gespeichert;
  das Gesamtvermögen monatlich als Verlauf.
- Am Start ist das Startgeld das gesamte Privatvermögen.

Lebensstil (vier Stufen; Kosten = Vielfaches des Akademiker-Monatslohns im Wohnsitzland;
Werte vorerst so, Nachjustieren nach dem Weltlauf in PE3; weitere private Ausgaben
vielleicht später):

| Stufe | Kosten je Monat | Zins eigener Firmen | Gehaltsforderung neuer Vorstände | Ausbildung der Kinder | Sterbechance |
| --- | --- | --- | --- | --- | --- |
| Bescheiden | 0,8 × | +0,5 Prozentpunkte | +5 % | ±0 Punkte | × 1,10 |
| Bürgerlich (Standard) | 1,5 × | ±0 | ±0 | +5 Punkte | × 1,00 |
| Gehoben | 4 × | −0,25 Prozentpunkte | −5 % | +10 Punkte | × 0,95 |
| Luxuriös | 10 × | −0,5 Prozentpunkte | −10 % | +15 Punkte | × 0,95 |

- Zins: Auf- oder Abschlag auf neue Bankkredite aller kontrollierten Firmen.
- Gehaltsforderung: Kandidaten auf Vorstandsstellen und CEO beim Einstellen.
- Ausbildung: Zuschlag auf das Mittel aller Fähigkeiten, wenn ein Kind mit 25 seine
  Managerkarte bekommt; maßgeblich ist der durchschnittliche Lebensstil seiner Kindheit.
- Wechsel höchstens einmal im Jahr; Wirkung ab dem Folgemonat.
- Guthaben aufgebraucht: Anliegen „Privatkonto leer“ (Gehalt erhöhen, Sonderausschüttung,
  Anteile verkaufen). Bei Fristablauf fällt die Stufe auf Bescheiden.

Spielende (ersetzt Lastenheft §11.3 Satz 2 und §11.2 letzter Satz):

- Pleite oder Übernahme der Hauptfirma beenden das Spiel nicht; die Person kann neu
  gründen, kaufen oder als Investor weiterspielen.
- Der Tod beendet es nicht, weil immer ein Erbe da ist.
- Das Spiel endet, wenn die Person keine Anteile mehr hält und ihr Guthaben unter der
  Gründungssumme der günstigsten Startform liegt; sonst am 31.12.2100.

## 9 Käufe, Verkäufe und Kapital zwischen Person und Firma

Geld fließt nur über diese Wege; einen freien Griff in die Firmenkasse gibt es nicht.

| Weg | Richtung | Regel |
| --- | --- | --- |
| Einlage | Person → Firma | Bei 100 % ohne Anteilsänderung; mit Mitgesellschaftern steigt der Anteil nach Firmenwert (Verwässerung wie SU2) |
| Gründungskapital | Person → neue Firma | Abschnitt 6 |
| Gesellschafterdarlehen | Person → Firma | Betrag, Zins, Laufzeit frei; Zins ist Aufwand; im Insolvenzfall nach den Bankkrediten |
| Gehalt, Dividende | Firma → Person | Abschnitt 7 |
| Kapitalrückzahlung | Firma → Person | Nur aus Einlagen, Liquiditätsreserve gewahrt |
| Kaufpreis | Person → Verkäufer | Anteile anderer Firmen und Start-ups |
| Verkaufserlös, Dividende | andere Firmen → Person | Anteile an fremden Firmen |

Kaufen:

- Neue Angebotsart **„Anteile“** in den Kaufangeboten; Bieter ist die Person oder eine
  ihrer Firmen. Annahme je Halter: `Holder::Company` entscheidet wie bei Bereichen (M31);
  `Private` und `Investors` nehmen an, wenn der Preis den Firmenwert um eine Prämie
  übersteigt (20–50 %, sinkt mit schlechter Ertragslage). Gegenangebote und Sperrfrist wie
  M30.
- Start-up-Anteile privat in Finanzierungsrunden (SU2), Regeln wie für Firmen.
- Neue Firmen gründen: Werkstatt, Handel oder Beteiligungs-, Investment-, Bankfirma
  (Lastenheft §17.3); Startkapital als Einlage.
- Börsennotierte Firmen ab K1 über die Börse.

Mehrere Firmen:

- Liste kontrollierter Firmen (über 50 % direkt oder über eine Kette eigener Firmen).
- Steuerung mit und ohne CEO wie in Abschnitt 5.
- Über 50 % an einem Start-up macht es zu einer eigenen Firma der Person, keine Tochter;
  Patente nutzen andere eigene Firmen über eine Lizenz.

Verkaufen:

- An eine Firma (Kaufangebot) oder an `Investors` sofort zum Firmenwert mit 5–20 %
  Abschlag.
- Steuer auf den Gewinn über dem Einstandswert mit `dividend_tax` des Wohnsitzlandes.
- Mehrheit verkauft: Die Firma wird KI-Firma (wie SU2), der Rest bleibt Beteiligung.
- Letzte Firma verkauft: Die Person spielt als Investor weiter.

## 10 Oberfläche und Bedienung

- **Neues Spiel:** Name der Person, Geburtsjahr, Familie, Startgeld (Zahlenfeld mit
  Untergrenze als Hinweis). Firmenname und Startform wandern in den Dialog Firma gründen.
- **Firma gründen (neu):** direkt nach dem Start und jederzeit aus der Ansicht Person.
- **Person (neu):** Steckbrief, Vermögen mit Verlauf, Lebensstil mit Wirkungen, Einnahmen
  und Ausgaben der letzten zwölf Monate, Lebenslauf. Aktionen: Firma gründen, Einlage,
  Darlehen, Lebensstil, Erbe bestimmen, Übergabe zu Lebzeiten.
- **Ohne Firma:** Übersicht mit Privatvermögen, Rang nach Vermögen und Weg zur Gründung;
  Ansichten, die eine Firma brauchen, sind ausgegraut.
- **Kopfzeile:** Firmenwahl ab zwei kontrollierten Firmen; daneben das Privatvermögen.
- **Organisation:** Spalten Alter und „Ruhestand in“; Filter „geht in den nächsten zwei
  Jahren“; Kinder als Kandidaten mit Kennzeichen.
- **Strategie:** Dividendenpolitik; Gehalt der Person als CEO.
- **Wettbewerb/Kaufangebote:** Angebotsart „Anteile“; Wahl des Bieters.
- **Meldungen:** Ruhestand angekündigt, Manager verstorben, Geburt, Erbfall, Dividende
  eingegangen, Privatkonto leer.
- **Rundenbericht:** Abschnitt Person mit Vermögensänderung und Ursachen (M27).
- **Einführung (M20):** beginnt mit der Gründung; später ein Schritt zur Ansicht Person.

## 11 Meilensteine

Reihenfolge: PE1–PE6 direkt nach dem Meilenstein, der bei der Übergabe läuft, vor den
übrigen Meilensteinen aus Architektur §4.1. K3 (Investoren) baut auf PE4/PE5 auf, W6
(Tochterfirmen) auf der Liste kontrollierter Firmen aus PE3.

| Nr. | Inhalt | Abnahme |
| --- | --- | --- |
| PE1 | Alter der Manager: Geburtsdatum, Altersverteilung, Alterswirkungen, Ruhestand mit Vorwarnung, Anliegen „Nachfolge regeln“, Pool altert; Zeitreihen `ruhestandsalter`, `lebenserwartung` | Weltlauf 1900–1960: kein Manager über 75 im Dienst, Altersverteilung je Ebene im Protokoll; Determinismus-Tests grün; alte Spielstände laden |
| PE2 | Spielerfigur: Person, Familie, Rollen, Lebenslauf, Felder im neuen Spiel, Ansicht Person (ohne Geld); Lastenheft §2 angepasst | Alter Spielstand lädt mit Standardperson; Kinder entstehen im Weltlauf plausibel |
| PE3 | Privatkonto und Spielstart: Startgeld als Eingabe, Start ohne Firma, Dialog Firma gründen, Gründungskosten, Liste kontrollierter Firmen statt `GameState.player`, Hauptfirma; Hauptbuch, Bewertung, Lebensstil mit Wirkungen, Gehalt als CEO, Einkommensteuer, Einlage, Gesellschafterdarlehen, Kapitalrückzahlung; Zeitreihen `einkommensteuer`, `sparzins` | Neues Spiel ohne Firma läuft Runden; Gründung unter der Untergrenze wird abgewiesen; Geldkreislauf geschlossen (Summe aller Konten plus Abflüsse stimmt jeden Monat); Einführung bis zum ersten Verkauf grün |
| PE4 | Dividenden: Politik, Grenzen, Verteilung nach Anteil, Quellensteuer, Konzernprivileg, Vorschlag des CEO, Regel für KI-Firmen | Weltlauf 100 KI-Firmen 1900–1930 ohne Pleitenwelle gegenüber vorher; Benchmark 1 000 KI-Firmen ohne deutlichen Leistungsverlust |
| PE5 | Käufe und Verkäufe der Person: Angebotsart Anteile, private Start-up-Anteile, weitere Gründungen, mehrere Firmen mit Firmenwahl, Steuerung mit und ohne CEO, Verkaufssteuer, neues Spielende | Szenario: Person kauft KI-Firma, steuert beide, verkauft eine; Szenario: Hauptfirma pleite, Spiel läuft als Investor weiter |
| PE6 | Tod, Erbe und Übergabe der Spielerfigur: Sterbetafel mit Lebensstil, Neffen-/Nichten-Regel, Erbschaftsteuer, Zwangsverkauf, Übergabe zu Lebzeiten; Zeitreihe `erbschaftsteuer` | Weltlauf 1900–2100 mit mindestens drei Generationen ohne Abbruch; Erbfall ohne Kind setzt Neffe oder Nichte ein |

Formeln wie bisher je Meilenstein in `docs/FORMELN.md`, Fortschritt in
`docs/FORTSCHRITT.md`.

## 12 Entscheidungen vom 07.10.2026

1. Sterblichkeit der Spielerfigur: immer an, keine Einstellung.
2. Ohne Kind erbt ein erzeugter Neffe oder eine Nichte.
3. Kinder arbeiten ab 25 als Manager in eigenen Firmen, ohne Kündigung und Abwerbung.
4. Lebensstil in vier Stufen mit Wirkung (Abschnitt 8).
5. Ohne CEO steuert der Spieler eine kontrollierte Firma selbst, mit CEO dieser.
6. Gehalt der Person bei Mitgesellschaftern höchstens doppelter Marktwert.
7. Abbau von Erkennen und Führung ab 65, langsam.
8. Einkommen-, Erbschaft- und Veräußerungsteuer je Land vereinfacht als ein Satz.
9. PE1–PE6 direkt im Anschluss umsetzen.
10. Start als Person ohne Firma; Startgeld vor Spielbeginn als Betrag eingeben.
11. Gründen, wann man will; vorher sind Runden als Investor möglich.
12. Lebensstil-Werte vorerst übernehmen; weitere private Ausgaben vielleicht später.
13. Hauptfirma ist die zuerst gegründete, vom Spieler wechselbar.
14. Gründungskosten 0,5 % der Einlage, mindestens ein Akademiker-Monatslohn, vorerst.
