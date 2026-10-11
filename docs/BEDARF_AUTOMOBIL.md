# Weltweiter Bedarf: Automobil

Erzeugt aus dem Spielkern (Stand Commit 9243c71, 46 Länder und Regionen). Für jedes Jahr wurde ein
neues Spiel mit diesem Startjahr begonnen (Seed 1, 100 KI-Firmen) und ein Monat gespielt; die Werte
stammen aus der Marktansicht jedes Landes (Produkt `automobil`).

- **Bedarf/Jahr**: Bedarf des ersten Spielmonats × 12, in Stück, **hochgerechnet auf reale Größe**.
  Mit 100 KI-Firmen rechnet das Spiel die Märkte im Maßstab 0,1 (`kimodell.yaml`:
  `firmen_bei_realer_groesse: 1000`); im Spiel siehst du also ein Zehntel dieser Zahlen.
- **Konsum / Staat**: Aufteilung nach den Bedarfsraten der Haushalte (fünf Einkommensgruppen) und des Staats.
- **Import / Export**: über Händler bezogen bzw. ins Ausland verkauft (gleiche Hochrechnung).
- **Ungedeckt**: Bedarf, den kein Anbieter bediente.
- **Preis**: Marktpreis im Land, USD Kaufkraft 2026 (Richtpreis 30.000 USD).

Produktdaten (`data/ketten/12_automobil.yaml`): Bedarfsklasse Luxus, Nutzungsdauer 10 Jahre,
höchstens 30 % Besitz je Kopf, Kaufschwelle 16, Preisempfindlichkeit 4, Einkommensempfindlichkeit 1,5;
dazu Staatsnachfrage (Post, Heer, Verwaltung).

## Überblick (Welt)

| Jahr | Bedarf/Jahr | Konsum | Staat | Ungedeckt | Größter Markt |
|---:|---:|---:|---:|---:|---|
| 1910 | 71.309 | 41.224 | 30.085 | 965 | Vereinigte Staaten (46 %) |
| 1930 | 117.318 | 40.606 | 76.712 | 15.210 | Vereinigte Staaten (46 %) |
| 1950 | 175.224 | 79.412 | 95.812 | 14.729 | Vereinigte Staaten (45 %) |
| 1970 | 551.512 | 244.857 | 306.655 | 48.172 | Vereinigte Staaten (37 %) |
| 1990 | 1.354.554 | 848.398 | 506.156 | 61.427 | Vereinigte Staaten (32 %) |
| 2010 | 2.285.955 | 1.472.108 | 813.847 | 0 | Vereinigte Staaten (30 %) |
| 2026 | 3.820.078 | 2.732.100 | 1.087.978 | 0 | Vereinigte Staaten (27 %) |

## Nach Kontinent (Bedarf/Jahr)

| Kontinent | 1910 | 1930 | 1950 | 1970 | 1990 | 2010 | 2026 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Nordamerika | 33.958 | 56.811 | 84.475 | 220.493 | 493.187 | 775.572 | 1.144.363 |
| Europa | 27.756 | 43.496 | 68.581 | 240.373 | 578.313 | 837.443 | 1.112.027 |
| Asien | 6.503 | 11.601 | 11.535 | 63.037 | 216.664 | 516.634 | 1.342.144 |
| Südamerika | 1.401 | 3.049 | 5.939 | 14.643 | 37.286 | 88.240 | 116.101 |
| Afrika | 689 | 995 | 1.742 | 5.938 | 12.005 | 33.386 | 56.367 |
| Ozeanien | 1.002 | 1.366 | 2.953 | 7.029 | 17.100 | 34.679 | 49.077 |

## Die 15 größten Märkte im Zeitverlauf (Bedarf/Jahr)

| Land/Region | 1910 | 1930 | 1950 | 1970 | 1990 | 2010 | 2026 |
|---|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | 32.566 | 54.210 | 78.661 | 201.643 | 435.971 | 677.990 | 1.016.530 |
| China | 1.917 | 4.120 | 2.986 | 1.984 | 7.721 | 105.420 | 560.230 |
| Deutschland | 4.809 | 6.950 | 5.581 | 36.948 | 98.741 | 148.379 | 181.683 |
| Japan | 1.112 | 2.749 | 2.030 | 30.048 | 127.676 | 135.527 | 155.054 |
| Russland | 1.672 | 1.879 | 14.379 | 63.431 | 107.861 | 102.090 | 141.810 |
| Vereinigtes Königreich und Irland | 8.501 | 11.047 | 19.468 | 23.432 | 54.393 | 91.334 | 125.105 |
| Frankreich | 3.665 | 7.900 | 7.341 | 24.960 | 66.401 | 97.775 | 114.028 |
| Indien, Nepal und Sri Lanka | 1.850 | 2.044 | 1.427 | 2.434 | 6.286 | 26.892 | 111.314 |
| Arabische Halbinsel | 33 | 48 | 99 | 5.746 | 27.936 | 91.852 | 101.947 |
| Italien und Malta | 1.964 | 3.244 | 3.541 | 21.082 | 73.060 | 93.727 | 97.587 |
| Nordeuropa und Baltikum | 952 | 2.116 | 4.257 | 12.807 | 32.526 | 58.583 | 81.716 |
| Benelux | 1.867 | 3.749 | 3.899 | 12.416 | 33.255 | 63.651 | 80.406 |
| Taiwan | 26 | 66 | 45 | 201 | 2.284 | 18.601 | 79.415 |
| Polen, Tschechien, Slowakei, Ungarn | 1.439 | 2.303 | 2.173 | 6.034 | 12.962 | 35.255 | 78.298 |
| Spanien und Portugal | 693 | 1.344 | 1.190 | 7.338 | 31.705 | 65.538 | 77.555 |

## 2026

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 1.016.530 | 26,6 % | 847.671 | 168.859 | 0 | 48.140 | 0 | 29.518 |
| China | Asien | 560.230 | 14,7 % | 389.950 | 170.280 | 0 | 2.019.770 | 0 | 25.691 |
| Deutschland | Europa | 181.683 | 4,8 % | 148.654 | 33.029 | 159.511 | 0 | 0 | 28.998 |
| Japan | Asien | 155.054 | 4,1 % | 120.839 | 34.215 | 155.054 | 0 | 0 | 28.057 |
| Russland | Europa | 141.810 | 3,7 % | 89.541 | 52.269 | 141.810 | 0 | 0 | 29.483 |
| Vereinigtes Königreich und Irland | Europa | 125.105 | 3,3 % | 101.948 | 23.157 | 125.105 | 0 | 0 | 28.460 |
| Frankreich | Europa | 114.028 | 3,0 % | 90.555 | 23.473 | 70.860 | 3.932 | 0 | 28.831 |
| Indien, Nepal und Sri Lanka | Asien | 111.314 | 2,9 % | 23.877 | 87.436 | 111.314 | 0 | 0 | 31.229 |
| Arabische Halbinsel | Asien | 101.947 | 2,7 % | 63.421 | 38.526 | 101.947 | 0 | 0 | 29.142 |
| Italien und Malta | Europa | 97.587 | 2,6 % | 78.245 | 19.342 | 97.587 | 0 | 0 | 28.445 |
| Nordeuropa und Baltikum | Europa | 81.716 | 2,1 % | 67.441 | 14.275 | 81.716 | 0 | 0 | 29.151 |
| Benelux | Europa | 80.406 | 2,1 % | 67.196 | 13.210 | 80.406 | 0 | 0 | 29.181 |
| Taiwan | Asien | 79.415 | 2,1 % | 67.908 | 11.507 | 79.415 | 0 | 0 | 29.528 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 78.298 | 2,0 % | 58.706 | 19.592 | 78.298 | 0 | 0 | 29.139 |
| Spanien und Portugal | Europa | 77.555 | 2,0 % | 60.210 | 17.344 | 77.555 | 0 | 0 | 28.449 |
| Türkei und Kaukasus | Asien | 74.891 | 2,0 % | 50.826 | 24.065 | 74.891 | 0 | 0 | 29.138 |
| Kanada | Nordamerika | 74.088 | 1,9 % | 60.618 | 13.470 | 74.088 | 0 | 0 | 28.478 |
| Malaysia, Singapur und Brunei | Asien | 61.113 | 1,6 % | 47.132 | 13.982 | 61.113 | 0 | 0 | 29.123 |
| Korea | Asien | 59.365 | 1,6 % | 40.669 | 18.696 | 59.365 | 0 | 0 | 29.103 |
| Brasilien | Südamerika | 58.054 | 1,5 % | 26.435 | 31.619 | 58.054 | 0 | 0 | 31.312 |
| Schweiz und Österreich | Europa | 50.318 | 1,3 % | 42.281 | 8.038 | 50.318 | 0 | 0 | 29.247 |
| Australien und Ozeanien | Ozeanien | 49.077 | 1,3 % | 37.257 | 11.820 | 49.077 | 0 | 0 | 28.096 |
| Südosteuropa | Europa | 46.817 | 1,2 % | 31.963 | 14.854 | 46.817 | 0 | 0 | 29.140 |
| Indonesien und Osttimor | Asien | 41.924 | 1,1 % | 16.340 | 25.585 | 41.924 | 0 | 0 | 29.130 |
| Mexiko | Nordamerika | 40.411 | 1,1 % | 21.229 | 19.182 | 40.411 | 0 | 0 | 29.524 |
| Ukraine und Belarus | Europa | 36.703 | 1,0 % | 2.212 | 34.491 | 36.703 | 0 | 0 | 29.137 |
| Argentinien, Paraguay und Uruguay | Südamerika | 25.651 | 0,7 % | 13.798 | 11.853 | 25.651 | 0 | 0 | 31.326 |
| Ägypten | Afrika | 21.376 | 0,6 % | 9.005 | 12.371 | 21.376 | 0 | 0 | 29.143 |
| Iran und Irak | Asien | 19.183 | 0,5 % | 7.200 | 11.984 | 19.183 | 0 | 0 | 29.132 |
| Festland-Südostasien | Asien | 17.797 | 0,5 % | 6.100 | 11.697 | 17.797 | 0 | 0 | 29.119 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 16.693 | 0,4 % | 7.002 | 9.691 | 16.693 | 0 | 0 | 29.188 |
| Peru, Bolivien und Chile | Südamerika | 15.703 | 0,4 % | 7.557 | 8.146 | 15.703 | 0 | 0 | 29.197 |
| Vietnam | Asien | 14.654 | 0,4 % | 5.625 | 9.030 | 14.654 | 0 | 0 | 29.113 |
| Mittelamerika und Karibik | Nordamerika | 13.333 | 0,3 % | 5.222 | 8.111 | 13.333 | 0 | 0 | 29.178 |
| Zentralasien und Mongolei | Asien | 12.358 | 0,3 % | 3.706 | 8.652 | 12.358 | 0 | 0 | 29.233 |
| Maghreb | Afrika | 11.342 | 0,3 % | 3.675 | 7.667 | 11.342 | 0 | 0 | 29.153 |
| Philippinen | Asien | 9.518 | 0,2 % | 2.736 | 6.782 | 9.518 | 0 | 0 | 29.116 |
| Bangladesch | Asien | 9.408 | 0,2 % | 1.999 | 7.409 | 9.408 | 0 | 0 | 29.118 |
| Levante und Israel | Asien | 7.887 | 0,2 % | 1.927 | 5.960 | 7.887 | 0 | 0 | 29.140 |
| Südliches Afrika | Afrika | 6.625 | 0,2 % | 1.236 | 5.389 | 6.625 | 0 | 0 | 29.178 |
| Pakistan | Asien | 6.086 | 0,2 % | 698 | 5.388 | 6.086 | 0 | 0 | 29.124 |
| Nigeria | Afrika | 5.882 | 0,2 % | 701 | 5.182 | 5.882 | 0 | 0 | 29.166 |
| Westafrika | Afrika | 3.778 | 0,1 % | 325 | 3.453 | 3.778 | 0 | 0 | 29.172 |
| Ostafrika | Afrika | 3.231 | 0,1 % | 235 | 2.996 | 3.231 | 0 | 0 | 29.159 |
| Kongo und Zentralafrika | Afrika | 2.593 | 0,1 % | 173 | 2.420 | 2.593 | 0 | 0 | 29.170 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 1.541 | 0,0 % | 59 | 1.482 | 1.541 | 0 | 0 | 29.155 |

## 2010

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 677.990 | 29,7 % | 565.227 | 112.763 | 0 | 1.393.226 | 0 | 29.589 |
| Deutschland | Europa | 148.379 | 6,5 % | 110.906 | 37.473 | 130.541 | 5.682 | 0 | 32.560 |
| Japan | Asien | 135.527 | 5,9 % | 91.766 | 43.761 | 135.527 | 0 | 0 | 32.363 |
| China | Asien | 105.420 | 4,6 % | 18.620 | 86.800 | 105.420 | 0 | 0 | 35.202 |
| Russland | Europa | 102.090 | 4,5 % | 56.505 | 45.584 | 102.090 | 0 | 0 | 34.608 |
| Frankreich | Europa | 97.775 | 4,3 % | 72.240 | 25.535 | 63.651 | 12.975 | 0 | 31.998 |
| Italien und Malta | Europa | 93.727 | 4,1 % | 68.001 | 25.726 | 93.727 | 0 | 0 | 32.603 |
| Arabische Halbinsel | Asien | 91.852 | 4,0 % | 31.141 | 60.711 | 91.852 | 0 | 0 | 33.582 |
| Vereinigtes Königreich und Irland | Europa | 91.334 | 4,0 % | 65.515 | 25.819 | 91.334 | 0 | 0 | 32.597 |
| Spanien und Portugal | Europa | 65.538 | 2,9 % | 44.752 | 20.787 | 65.538 | 0 | 0 | 32.599 |
| Benelux | Europa | 63.651 | 2,8 % | 48.299 | 15.352 | 63.651 | 0 | 0 | 33.156 |
| Kanada | Nordamerika | 59.078 | 2,6 % | 45.656 | 13.422 | 59.078 | 0 | 0 | 31.093 |
| Nordeuropa und Baltikum | Europa | 58.583 | 2,6 % | 42.951 | 15.632 | 58.583 | 0 | 0 | 33.159 |
| Brasilien | Südamerika | 43.988 | 1,9 % | 14.064 | 29.924 | 43.988 | 0 | 0 | 36.232 |
| Schweiz und Österreich | Europa | 42.969 | 1,9 % | 33.337 | 9.632 | 42.969 | 0 | 0 | 33.198 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 35.255 | 1,5 % | 19.159 | 16.096 | 35.255 | 0 | 0 | 33.163 |
| Australien und Ozeanien | Ozeanien | 34.679 | 1,5 % | 22.744 | 11.935 | 34.679 | 0 | 0 | 32.642 |
| Mexiko | Nordamerika | 31.456 | 1,4 % | 15.534 | 15.922 | 31.456 | 0 | 0 | 31.106 |
| Korea | Asien | 30.229 | 1,3 % | 14.518 | 15.712 | 30.229 | 0 | 0 | 33.580 |
| Indien, Nepal und Sri Lanka | Asien | 26.892 | 1,2 % | 1.642 | 25.250 | 26.892 | 0 | 0 | 35.210 |
| Südosteuropa | Europa | 26.802 | 1,2 % | 12.826 | 13.976 | 26.802 | 0 | 0 | 33.167 |
| Malaysia, Singapur und Brunei | Asien | 26.127 | 1,1 % | 15.704 | 10.424 | 26.127 | 0 | 0 | 33.605 |
| Argentinien, Paraguay und Uruguay | Südamerika | 21.916 | 1,0 % | 9.328 | 12.588 | 21.916 | 0 | 0 | 36.242 |
| Türkei und Kaukasus | Asien | 20.515 | 0,9 % | 7.703 | 12.812 | 20.515 | 0 | 0 | 33.566 |
| Taiwan | Asien | 18.601 | 0,8 % | 11.179 | 7.422 | 18.601 | 0 | 0 | 33.589 |
| Iran und Irak | Asien | 14.043 | 0,6 % | 6.786 | 7.257 | 14.043 | 0 | 0 | 26.840 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 13.748 | 0,6 % | 4.333 | 9.415 | 13.748 | 0 | 0 | 33.538 |
| Indonesien und Osttimor | Asien | 13.596 | 0,6 % | 2.128 | 11.468 | 13.596 | 0 | 0 | 33.613 |
| Ukraine und Belarus | Europa | 11.340 | 0,5 % | 3.893 | 7.447 | 11.340 | 0 | 0 | 33.561 |
| Ägypten | Afrika | 10.113 | 0,4 % | 2.535 | 7.578 | 10.113 | 0 | 0 | 33.571 |
| Festland-Südostasien | Asien | 9.998 | 0,4 % | 1.919 | 8.079 | 9.998 | 0 | 0 | 33.600 |
| Peru, Bolivien und Chile | Südamerika | 8.589 | 0,4 % | 2.677 | 5.912 | 8.589 | 0 | 0 | 33.549 |
| Maghreb | Afrika | 8.387 | 0,4 % | 1.930 | 6.457 | 8.387 | 0 | 0 | 33.556 |
| Levante und Israel | Asien | 7.189 | 0,3 % | 1.869 | 5.320 | 7.189 | 0 | 0 | 33.571 |
| Mittelamerika und Karibik | Nordamerika | 7.048 | 0,3 % | 1.615 | 5.432 | 7.048 | 0 | 0 | 33.533 |
| Südliches Afrika | Afrika | 5.640 | 0,2 % | 836 | 4.804 | 5.640 | 0 | 0 | 33.592 |
| Zentralasien und Mongolei | Asien | 4.820 | 0,2 % | 706 | 4.115 | 4.820 | 0 | 0 | 33.687 |
| Nigeria | Afrika | 3.867 | 0,2 % | 321 | 3.546 | 3.867 | 0 | 0 | 33.571 |
| Vietnam | Asien | 3.410 | 0,1 % | 418 | 2.992 | 3.410 | 0 | 0 | 33.596 |
| Pakistan | Asien | 3.187 | 0,1 % | 179 | 3.008 | 3.187 | 0 | 0 | 33.586 |
| Philippinen | Asien | 2.977 | 0,1 % | 317 | 2.661 | 2.977 | 0 | 0 | 33.599 |
| Bangladesch | Asien | 2.250 | 0,1 % | 119 | 2.131 | 2.250 | 0 | 0 | 33.596 |
| Kongo und Zentralafrika | Afrika | 1.919 | 0,1 % | 104 | 1.815 | 1.919 | 0 | 0 | 33.581 |
| Westafrika | Afrika | 1.417 | 0,1 % | 51 | 1.367 | 1.417 | 0 | 0 | 33.571 |
| Ostafrika | Afrika | 1.332 | 0,1 % | 43 | 1.289 | 1.332 | 0 | 0 | 33.597 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 711 | 0,0 % | 13 | 698 | 711 | 0 | 0 | 33.593 |

## 1990

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 435.971 | 32,2 % | 371.637 | 64.334 | 0 | 644.201 | 0 | 29.279 |
| Japan | Asien | 127.676 | 9,4 % | 91.550 | 36.126 | 127.676 | 0 | 0 | 32.062 |
| Russland | Europa | 107.861 | 8,0 % | 10.135 | 97.726 | 57.579 | 0 | 50.282 | 50.877 |
| Deutschland | Europa | 98.741 | 7,3 % | 72.795 | 25.946 | 79.704 | 4.483 | 0 | 32.189 |
| Italien und Malta | Europa | 73.060 | 5,4 % | 53.153 | 19.907 | 73.060 | 0 | 0 | 32.812 |
| Frankreich | Europa | 66.401 | 4,9 % | 50.402 | 15.999 | 32.705 | 13.403 | 0 | 30.938 |
| Vereinigtes Königreich und Irland | Europa | 54.393 | 4,0 % | 36.648 | 17.744 | 54.393 | 0 | 0 | 33.260 |
| Kanada | Nordamerika | 36.804 | 2,7 % | 28.613 | 8.191 | 36.804 | 0 | 0 | 30.800 |
| Benelux | Europa | 33.255 | 2,5 % | 22.071 | 11.185 | 33.255 | 0 | 0 | 35.608 |
| Nordeuropa und Baltikum | Europa | 32.526 | 2,4 % | 20.238 | 12.288 | 32.526 | 0 | 0 | 35.635 |
| Spanien und Portugal | Europa | 31.705 | 2,3 % | 19.552 | 12.153 | 31.705 | 0 | 0 | 32.976 |
| Arabische Halbinsel | Asien | 27.936 | 2,1 % | 5.573 | 22.363 | 27.936 | 0 | 0 | 35.675 |
| Schweiz und Österreich | Europa | 26.985 | 2,0 % | 19.348 | 7.637 | 26.985 | 0 | 0 | 35.691 |
| Ukraine und Belarus | Europa | 25.466 | 1,9 % | 5.205 | 20.261 | 25.466 | 0 | 0 | 35.645 |
| Brasilien | Südamerika | 21.305 | 1,6 % | 3.577 | 17.729 | 21.305 | 0 | 0 | 42.855 |
| Mexiko | Nordamerika | 17.648 | 1,3 % | 6.353 | 11.295 | 17.648 | 0 | 0 | 35.610 |
| Australien und Ozeanien | Ozeanien | 17.100 | 1,3 % | 9.304 | 7.795 | 17.100 | 0 | 0 | 35.715 |
| Südosteuropa | Europa | 14.956 | 1,1 % | 4.809 | 10.148 | 14.956 | 0 | 0 | 35.647 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 12.962 | 1,0 % | 4.160 | 8.801 | 12.962 | 0 | 0 | 35.640 |
| Türkei und Kaukasus | Asien | 8.018 | 0,6 % | 1.911 | 6.107 | 8.018 | 0 | 0 | 35.654 |
| Argentinien, Paraguay und Uruguay | Südamerika | 7.797 | 0,6 % | 2.368 | 5.429 | 7.797 | 0 | 0 | 36.857 |
| China | Asien | 7.721 | 0,6 % | 58 | 7.663 | 1.559 | 0 | 6.162 | 47.493 |
| Zentralasien und Mongolei | Asien | 6.343 | 0,5 % | 415 | 5.928 | 6.343 | 0 | 0 | 35.832 |
| Indien, Nepal und Sri Lanka | Asien | 6.286 | 0,5 % | 16 | 6.270 | 1.303 | 0 | 4.983 | 62.981 |
| Korea | Asien | 6.085 | 0,4 % | 1.287 | 4.798 | 6.085 | 0 | 0 | 35.676 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 5.797 | 0,4 % | 1.259 | 4.537 | 5.797 | 0 | 0 | 35.618 |
| Malaysia, Singapur und Brunei | Asien | 4.905 | 0,4 % | 1.794 | 3.111 | 4.905 | 0 | 0 | 35.709 |
| Iran und Irak | Asien | 4.584 | 0,3 % | 733 | 3.851 | 4.584 | 0 | 0 | 35.667 |
| Levante und Israel | Asien | 4.453 | 0,3 % | 293 | 4.161 | 4.453 | 0 | 0 | 35.660 |
| Indonesien und Osttimor | Asien | 4.123 | 0,3 % | 290 | 3.833 | 4.123 | 0 | 0 | 35.721 |
| Maghreb | Afrika | 3.164 | 0,2 % | 441 | 2.723 | 3.164 | 0 | 0 | 35.639 |
| Südliches Afrika | Afrika | 2.984 | 0,2 % | 337 | 2.646 | 2.984 | 0 | 0 | 35.690 |
| Mittelamerika und Karibik | Nordamerika | 2.764 | 0,2 % | 387 | 2.377 | 2.764 | 0 | 0 | 35.612 |
| Festland-Südostasien | Asien | 2.639 | 0,2 % | 213 | 2.425 | 2.639 | 0 | 0 | 35.702 |
| Ägypten | Afrika | 2.528 | 0,2 % | 306 | 2.223 | 2.528 | 0 | 0 | 35.660 |
| Peru, Bolivien und Chile | Südamerika | 2.386 | 0,2 % | 373 | 2.014 | 2.386 | 0 | 0 | 35.634 |
| Taiwan | Asien | 2.284 | 0,2 % | 525 | 1.759 | 2.284 | 0 | 0 | 35.688 |
| Pakistan | Asien | 1.345 | 0,1 % | 52 | 1.293 | 1.345 | 0 | 0 | 35.681 |
| Philippinen | Asien | 1.294 | 0,1 % | 89 | 1.205 | 1.294 | 0 | 0 | 35.700 |
| Nigeria | Afrika | 1.092 | 0,1 % | 45 | 1.047 | 1.092 | 0 | 0 | 35.661 |
| Kongo und Zentralafrika | Afrika | 776 | 0,1 % | 30 | 745 | 776 | 0 | 0 | 35.675 |
| Westafrika | Afrika | 667 | 0,0 % | 19 | 649 | 667 | 0 | 0 | 35.660 |
| Ostafrika | Afrika | 545 | 0,0 % | 12 | 533 | 545 | 0 | 0 | 35.697 |
| Bangladesch | Asien | 510 | 0,0 % | 8 | 502 | 510 | 0 | 0 | 35.701 |
| Vietnam | Asien | 462 | 0,0 % | 12 | 450 | 462 | 0 | 0 | 35.701 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 249 | 0,0 % | 3 | 247 | 249 | 0 | 0 | 35.698 |

## 1970

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 201.643 | 36,6 % | 108.770 | 92.872 | 201.643 | 0 | 0 | 33.878 |
| Russland | Europa | 63.431 | 11,5 % | 3.328 | 60.103 | 23.028 | 0 | 40.403 | 50.174 |
| Deutschland | Europa | 36.948 | 6,7 % | 25.878 | 11.070 | 18.268 | 4.840 | 0 | 30.397 |
| Japan | Asien | 30.048 | 5,4 % | 14.865 | 15.183 | 30.048 | 0 | 0 | 34.449 |
| Frankreich | Europa | 24.960 | 4,5 % | 17.982 | 6.979 | 0 | 23.750 | 0 | 30.221 |
| Vereinigtes Königreich und Irland | Europa | 23.432 | 4,2 % | 12.957 | 10.475 | 23.432 | 0 | 0 | 34.664 |
| Italien und Malta | Europa | 21.082 | 3,8 % | 14.314 | 6.768 | 21.082 | 0 | 0 | 30.393 |
| Ukraine und Belarus | Europa | 16.641 | 3,0 % | 1.819 | 14.823 | 16.641 | 0 | 0 | 36.991 |
| Nordeuropa und Baltikum | Europa | 12.807 | 2,3 % | 6.523 | 6.285 | 12.807 | 0 | 0 | 36.984 |
| Kanada | Nordamerika | 12.559 | 2,3 % | 7.936 | 4.623 | 12.559 | 0 | 0 | 34.399 |
| Benelux | Europa | 12.416 | 2,3 % | 9.006 | 3.409 | 12.416 | 0 | 0 | 30.386 |
| Schweiz und Österreich | Europa | 10.109 | 1,8 % | 6.147 | 3.962 | 10.109 | 0 | 0 | 37.084 |
| Zentralasien und Mongolei | Asien | 7.618 | 1,4 % | 518 | 7.100 | 7.618 | 0 | 0 | 37.369 |
| Spanien und Portugal | Europa | 7.338 | 1,3 % | 2.238 | 5.099 | 7.338 | 0 | 0 | 38.295 |
| Australien und Ozeanien | Ozeanien | 7.029 | 1,3 % | 2.718 | 4.312 | 7.029 | 0 | 0 | 40.497 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 6.034 | 1,1 % | 1.497 | 4.538 | 6.034 | 0 | 0 | 36.983 |
| Argentinien, Paraguay und Uruguay | Südamerika | 5.904 | 1,1 % | 1.179 | 4.725 | 3.897 | 0 | 2.007 | 45.700 |
| Arabische Halbinsel | Asien | 5.746 | 1,0 % | 1.488 | 4.259 | 5.746 | 0 | 0 | 37.033 |
| Südosteuropa | Europa | 5.175 | 0,9 % | 1.093 | 4.081 | 5.175 | 0 | 0 | 36.990 |
| Mexiko | Nordamerika | 4.973 | 0,9 % | 992 | 3.981 | 4.973 | 0 | 0 | 40.389 |
| Iran und Irak | Asien | 4.195 | 0,8 % | 1.112 | 3.083 | 4.195 | 0 | 0 | 37.020 |
| Brasilien | Südamerika | 4.007 | 0,7 % | 203 | 3.804 | 1.997 | 0 | 2.010 | 50.289 |
| Levante und Israel | Asien | 3.846 | 0,7 % | 120 | 3.725 | 3.846 | 0 | 0 | 37.009 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 2.952 | 0,5 % | 653 | 2.299 | 2.952 | 0 | 0 | 37.080 |
| Türkei und Kaukasus | Asien | 2.599 | 0,5 % | 442 | 2.156 | 2.599 | 0 | 0 | 37.000 |
| Indien, Nepal und Sri Lanka | Asien | 2.434 | 0,4 % | 11 | 2.423 | 437 | 0 | 1.997 | 52.913 |
| China | Asien | 1.984 | 0,4 % | 5 | 1.978 | 229 | 0 | 1.755 | 50.325 |
| Peru, Bolivien und Chile | Südamerika | 1.780 | 0,3 % | 350 | 1.430 | 1.780 | 0 | 0 | 37.101 |
| Ägypten | Afrika | 1.686 | 0,3 % | 12 | 1.674 | 1.686 | 0 | 0 | 37.014 |
| Südliches Afrika | Afrika | 1.496 | 0,3 % | 187 | 1.309 | 1.496 | 0 | 0 | 37.091 |
| Mittelamerika und Karibik | Nordamerika | 1.319 | 0,2 % | 162 | 1.156 | 1.319 | 0 | 0 | 37.086 |
| Maghreb | Afrika | 909 | 0,2 % | 89 | 820 | 909 | 0 | 0 | 36.986 |
| Vietnam | Asien | 816 | 0,1 % | 2 | 814 | 816 | 0 | 0 | 37.109 |
| Indonesien und Osttimor | Asien | 709 | 0,1 % | 18 | 691 | 709 | 0 | 0 | 37.129 |
| Nigeria | Afrika | 582 | 0,1 % | 26 | 555 | 582 | 0 | 0 | 37.030 |
| Philippinen | Asien | 557 | 0,1 % | 34 | 523 | 557 | 0 | 0 | 37.111 |
| Malaysia, Singapur und Brunei | Asien | 550 | 0,1 % | 84 | 466 | 550 | 0 | 0 | 37.104 |
| Festland-Südostasien | Asien | 510 | 0,1 % | 16 | 494 | 510 | 0 | 0 | 37.099 |
| Pakistan | Asien | 491 | 0,1 % | 6 | 485 | 491 | 0 | 0 | 37.057 |
| Korea | Asien | 470 | 0,1 % | 19 | 451 | 470 | 0 | 0 | 37.090 |
| Westafrika | Afrika | 425 | 0,1 % | 16 | 409 | 425 | 0 | 0 | 37.037 |
| Kongo und Zentralafrika | Afrika | 412 | 0,1 % | 18 | 394 | 412 | 0 | 0 | 37.053 |
| Bangladesch | Asien | 264 | 0,0 % | 4 | 260 | 264 | 0 | 0 | 37.090 |
| Ostafrika | Afrika | 260 | 0,0 % | 6 | 253 | 260 | 0 | 0 | 37.074 |
| Taiwan | Asien | 201 | 0,0 % | 11 | 190 | 201 | 0 | 0 | 37.099 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 169 | 0,0 % | 3 | 166 | 169 | 0 | 0 | 37.063 |

## 1950

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 78.661 | 44,9 % | 53.839 | 24.822 | 78.661 | 0 | 0 | 33.049 |
| Vereinigtes Königreich und Irland | Europa | 19.468 | 11,1 % | 6.160 | 13.309 | 19.468 | 0 | 0 | 35.619 |
| Russland | Europa | 14.379 | 8,2 % | 370 | 14.009 | 2.989 | 0 | 11.390 | 45.907 |
| Frankreich | Europa | 7.341 | 4,2 % | 4.769 | 2.572 | 0 | 38.515 | 0 | 28.669 |
| Deutschland | Europa | 5.581 | 3,2 % | 2.917 | 2.665 | 4 | 18.123 | 0 | 27.469 |
| Kanada | Nordamerika | 4.354 | 2,5 % | 2.576 | 1.778 | 4.354 | 0 | 0 | 33.302 |
| Nordeuropa und Baltikum | Europa | 4.257 | 2,4 % | 1.607 | 2.649 | 4.257 | 0 | 0 | 37.068 |
| Benelux | Europa | 3.899 | 2,2 % | 1.614 | 2.285 | 3.899 | 0 | 0 | 36.870 |
| Ukraine und Belarus | Europa | 3.802 | 2,2 % | 146 | 3.656 | 3.802 | 0 | 0 | 36.923 |
| Italien und Malta | Europa | 3.541 | 2,0 % | 878 | 2.662 | 3.541 | 0 | 0 | 36.880 |
| Argentinien, Paraguay und Uruguay | Südamerika | 3.160 | 1,8 % | 722 | 2.438 | 3.160 | 0 | 0 | 43.614 |
| China | Asien | 2.986 | 1,7 % | 6 | 2.979 | 176 | 0 | 2.810 | 45.964 |
| Australien und Ozeanien | Ozeanien | 2.953 | 1,7 % | 1.294 | 1.659 | 2.953 | 0 | 0 | 37.049 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 2.173 | 1,2 % | 366 | 1.807 | 2.173 | 0 | 0 | 36.916 |
| Japan | Asien | 2.030 | 1,2 % | 253 | 1.777 | 2.030 | 0 | 0 | 35.098 |
| Schweiz und Österreich | Europa | 1.964 | 1,1 % | 717 | 1.247 | 1.964 | 0 | 0 | 37.448 |
| Zentralasien und Mongolei | Asien | 1.850 | 1,1 % | 57 | 1.793 | 1.850 | 0 | 0 | 37.440 |
| Indien, Nepal und Sri Lanka | Asien | 1.427 | 0,8 % | 24 | 1.403 | 1.427 | 0 | 0 | 38.678 |
| Spanien und Portugal | Europa | 1.190 | 0,7 % | 135 | 1.056 | 1.190 | 0 | 0 | 38.697 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 1.112 | 0,6 % | 223 | 889 | 1.112 | 0 | 0 | 36.917 |
| Südosteuropa | Europa | 986 | 0,6 % | 83 | 903 | 986 | 0 | 0 | 36.926 |
| Mexiko | Nordamerika | 964 | 0,6 % | 140 | 824 | 964 | 0 | 0 | 37.528 |
| Brasilien | Südamerika | 837 | 0,5 % | 28 | 809 | 307 | 0 | 530 | 46.585 |
| Peru, Bolivien und Chile | Südamerika | 830 | 0,5 % | 151 | 679 | 830 | 0 | 0 | 36.951 |
| Iran und Irak | Asien | 525 | 0,3 % | 59 | 466 | 525 | 0 | 0 | 36.936 |
| Türkei und Kaukasus | Asien | 504 | 0,3 % | 43 | 461 | 504 | 0 | 0 | 36.933 |
| Mittelamerika und Karibik | Nordamerika | 495 | 0,3 % | 48 | 447 | 495 | 0 | 0 | 36.924 |
| Levante und Israel | Asien | 468 | 0,3 % | 29 | 439 | 468 | 0 | 0 | 36.900 |
| Südliches Afrika | Afrika | 457 | 0,3 % | 37 | 420 | 457 | 0 | 0 | 36.925 |
| Indonesien und Osttimor | Asien | 379 | 0,2 % | 12 | 367 | 379 | 0 | 0 | 36.960 |
| Maghreb | Afrika | 323 | 0,2 % | 23 | 300 | 323 | 0 | 0 | 36.891 |
| Nigeria | Afrika | 255 | 0,1 % | 10 | 245 | 255 | 0 | 0 | 36.916 |
| Korea | Asien | 243 | 0,1 % | 2 | 240 | 243 | 0 | 0 | 36.911 |
| Malaysia, Singapur und Brunei | Asien | 183 | 0,1 % | 21 | 161 | 183 | 0 | 0 | 36.922 |
| Philippinen | Asien | 174 | 0,1 % | 9 | 165 | 174 | 0 | 0 | 36.934 |
| Festland-Südostasien | Asien | 171 | 0,1 % | 4 | 168 | 171 | 0 | 0 | 36.928 |
| Westafrika | Afrika | 171 | 0,1 % | 5 | 166 | 171 | 0 | 0 | 36.915 |
| Ägypten | Afrika | 160 | 0,1 % | 6 | 154 | 160 | 0 | 0 | 36.902 |
| Bangladesch | Asien | 157 | 0,1 % | 3 | 154 | 157 | 0 | 0 | 36.898 |
| Kongo und Zentralafrika | Afrika | 155 | 0,1 % | 5 | 150 | 155 | 0 | 0 | 36.866 |
| Pakistan | Asien | 154 | 0,1 % | 3 | 150 | 154 | 0 | 0 | 36.895 |
| Vietnam | Asien | 141 | 0,1 % | 2 | 138 | 141 | 0 | 0 | 36.916 |
| Ostafrika | Afrika | 126 | 0,1 % | 3 | 122 | 126 | 0 | 0 | 36.890 |
| Arabische Halbinsel | Asien | 99 | 0,1 % | 7 | 93 | 99 | 0 | 0 | 36.913 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 96 | 0,1 % | 2 | 94 | 96 | 0 | 0 | 36.869 |
| Taiwan | Asien | 45 | 0,0 % | 1 | 44 | 45 | 0 | 0 | 36.920 |

## 1930

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 54.210 | 46,2 % | 19.506 | 34.705 | 41.532 | 0 | 12.679 | 46.856 |
| Vereinigtes Königreich und Irland | Europa | 11.047 | 9,4 % | 5.575 | 5.472 | 11.047 | 0 | 0 | 35.260 |
| Frankreich | Europa | 7.900 | 6,7 % | 5.402 | 2.498 | 0 | 37.987 | 0 | 28.771 |
| Deutschland | Europa | 6.950 | 5,9 % | 4.111 | 2.839 | 0 | 16.570 | 0 | 28.449 |
| China | Asien | 4.120 | 3,5 % | 105 | 4.014 | 4.120 | 0 | 0 | 33.059 |
| Benelux | Europa | 3.749 | 3,2 % | 1.477 | 2.273 | 3.749 | 0 | 0 | 39.898 |
| Italien und Malta | Europa | 3.244 | 2,8 % | 386 | 2.858 | 2.554 | 0 | 690 | 45.075 |
| Japan | Asien | 2.749 | 2,3 % | 364 | 2.385 | 2.749 | 0 | 0 | 40.581 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 2.303 | 2,0 % | 329 | 1.974 | 2.303 | 0 | 0 | 39.806 |
| Nordeuropa und Baltikum | Europa | 2.116 | 1,8 % | 526 | 1.590 | 2.116 | 0 | 0 | 39.834 |
| Indien, Nepal und Sri Lanka | Asien | 2.044 | 1,7 % | 63 | 1.981 | 2.044 | 0 | 0 | 38.173 |
| Kanada | Nordamerika | 1.994 | 1,7 % | 774 | 1.219 | 1.994 | 0 | 0 | 38.734 |
| Russland | Europa | 1.879 | 1,6 % | 74 | 1.805 | 720 | 0 | 1.159 | 48.102 |
| Argentinien, Paraguay und Uruguay | Südamerika | 1.851 | 1,6 % | 512 | 1.339 | 1.851 | 0 | 0 | 40.765 |
| Schweiz und Österreich | Europa | 1.793 | 1,5 % | 583 | 1.210 | 1.793 | 0 | 0 | 40.026 |
| Australien und Ozeanien | Ozeanien | 1.366 | 1,2 % | 302 | 1.064 | 1.366 | 0 | 0 | 44.700 |
| Spanien und Portugal | Europa | 1.344 | 1,1 % | 100 | 1.244 | 662 | 0 | 682 | 47.911 |
| Südosteuropa | Europa | 770 | 0,7 % | 53 | 716 | 770 | 0 | 0 | 39.825 |
| Indonesien und Osttimor | Asien | 626 | 0,5 % | 33 | 593 | 626 | 0 | 0 | 39.981 |
| Peru, Bolivien und Chile | Südamerika | 580 | 0,5 % | 88 | 492 | 580 | 0 | 0 | 39.984 |
| Ukraine und Belarus | Europa | 400 | 0,3 % | 18 | 381 | 400 | 0 | 0 | 39.826 |
| Mexiko | Nordamerika | 342 | 0,3 % | 27 | 315 | 342 | 0 | 0 | 40.712 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 323 | 0,3 % | 30 | 293 | 323 | 0 | 0 | 39.942 |
| Brasilien | Südamerika | 295 | 0,3 % | 10 | 285 | 295 | 0 | 0 | 42.618 |
| Mittelamerika und Karibik | Nordamerika | 265 | 0,2 % | 18 | 247 | 265 | 0 | 0 | 39.949 |
| Korea | Asien | 220 | 0,2 % | 10 | 210 | 220 | 0 | 0 | 39.912 |
| Türkei und Kaukasus | Asien | 217 | 0,2 % | 10 | 207 | 217 | 0 | 0 | 39.833 |
| Zentralasien und Mongolei | Asien | 215 | 0,2 % | 8 | 207 | 215 | 0 | 0 | 40.661 |
| Philippinen | Asien | 212 | 0,2 % | 15 | 196 | 212 | 0 | 0 | 39.954 |
| Festland-Südostasien | Asien | 207 | 0,2 % | 6 | 201 | 207 | 0 | 0 | 39.929 |
| Iran und Irak | Asien | 199 | 0,2 % | 11 | 188 | 199 | 0 | 0 | 39.844 |
| Maghreb | Afrika | 193 | 0,2 % | 10 | 183 | 193 | 0 | 0 | 39.834 |
| Südliches Afrika | Afrika | 167 | 0,1 % | 6 | 160 | 167 | 0 | 0 | 39.954 |
| Malaysia, Singapur und Brunei | Asien | 163 | 0,1 % | 20 | 144 | 163 | 0 | 0 | 39.945 |
| Pakistan | Asien | 159 | 0,1 % | 4 | 154 | 159 | 0 | 0 | 39.842 |
| Nigeria | Afrika | 152 | 0,1 % | 4 | 148 | 152 | 0 | 0 | 39.860 |
| Ägypten | Afrika | 140 | 0,1 % | 6 | 135 | 140 | 0 | 0 | 39.842 |
| Bangladesch | Asien | 138 | 0,1 % | 3 | 135 | 138 | 0 | 0 | 39.901 |
| Westafrika | Afrika | 119 | 0,1 % | 3 | 116 | 119 | 0 | 0 | 39.866 |
| Levante und Israel | Asien | 115 | 0,1 % | 10 | 104 | 115 | 0 | 0 | 39.841 |
| Vietnam | Asien | 102 | 0,1 % | 3 | 100 | 102 | 0 | 0 | 39.926 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 83 | 0,1 % | 1 | 82 | 83 | 0 | 0 | 39.867 |
| Ostafrika | Afrika | 78 | 0,1 % | 1 | 77 | 78 | 0 | 0 | 39.912 |
| Taiwan | Asien | 66 | 0,1 % | 4 | 62 | 66 | 0 | 0 | 39.931 |
| Kongo und Zentralafrika | Afrika | 62 | 0,1 % | 1 | 62 | 62 | 0 | 0 | 39.893 |
| Arabische Halbinsel | Asien | 48 | 0,0 % | 2 | 46 | 48 | 0 | 0 | 39.851 |

## 1910

| Land/Region | Kontinent | Bedarf/Jahr | Anteil | Konsum | Staat | Import | Export | Ungedeckt | Preis USD |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vereinigte Staaten | Nordamerika | 32.566 | 45,7 % | 25.409 | 7.157 | 0 | 0 | 0 | 29.451 |
| Vereinigtes Königreich und Irland | Europa | 8.501 | 11,9 % | 5.659 | 2.842 | 8.501 | 0 | 0 | 29.400 |
| Deutschland | Europa | 4.809 | 6,7 % | 3.194 | 1.615 | 0 | 9.672 | 0 | 26.818 |
| Frankreich | Europa | 3.665 | 5,1 % | 2.375 | 1.290 | 0 | 17.826 | 0 | 26.788 |
| Italien und Malta | Europa | 1.964 | 2,8 % | 544 | 1.420 | 1.964 | 0 | 0 | 34.955 |
| China | Asien | 1.917 | 2,7 % | 124 | 1.793 | 1.917 | 0 | 0 | 30.183 |
| Benelux | Europa | 1.867 | 2,6 % | 902 | 965 | 1.867 | 0 | 0 | 33.669 |
| Indien, Nepal und Sri Lanka | Asien | 1.850 | 2,6 % | 127 | 1.723 | 1.850 | 0 | 0 | 31.673 |
| Russland | Europa | 1.672 | 2,3 % | 46 | 1.626 | 707 | 0 | 965 | 55.848 |
| Polen, Tschechien, Slowakei, Ungarn | Europa | 1.439 | 2,0 % | 289 | 1.151 | 1.439 | 0 | 0 | 33.678 |
| Schweiz und Österreich | Europa | 1.351 | 1,9 % | 614 | 737 | 1.351 | 0 | 0 | 33.911 |
| Japan | Asien | 1.112 | 1,6 % | 146 | 966 | 1.112 | 0 | 0 | 36.298 |
| Australien und Ozeanien | Ozeanien | 1.002 | 1,4 % | 427 | 575 | 1.002 | 0 | 0 | 35.916 |
| Kanada | Nordamerika | 957 | 1,3 % | 405 | 552 | 957 | 0 | 0 | 35.521 |
| Nordeuropa und Baltikum | Europa | 952 | 1,3 % | 266 | 686 | 952 | 0 | 0 | 33.684 |
| Argentinien, Paraguay und Uruguay | Südamerika | 937 | 1,3 % | 306 | 631 | 937 | 0 | 0 | 38.251 |
| Spanien und Portugal | Europa | 693 | 1,0 % | 60 | 633 | 693 | 0 | 0 | 42.015 |
| Südosteuropa | Europa | 524 | 0,7 % | 62 | 461 | 524 | 0 | 0 | 33.698 |
| Ukraine und Belarus | Europa | 319 | 0,4 % | 29 | 289 | 319 | 0 | 0 | 33.700 |
| Indonesien und Osttimor | Asien | 314 | 0,4 % | 21 | 293 | 314 | 0 | 0 | 33.942 |
| Mexiko | Nordamerika | 300 | 0,4 % | 28 | 272 | 300 | 0 | 0 | 39.429 |
| Türkei und Kaukasus | Asien | 267 | 0,4 % | 16 | 251 | 267 | 0 | 0 | 33.719 |
| Peru, Bolivien und Chile | Südamerika | 253 | 0,4 % | 47 | 205 | 253 | 0 | 0 | 33.926 |
| Zentralasien und Mongolei | Asien | 153 | 0,2 % | 10 | 142 | 153 | 0 | 0 | 34.577 |
| Mittelamerika und Karibik | Nordamerika | 135 | 0,2 % | 14 | 121 | 135 | 0 | 0 | 33.897 |
| Festland-Südostasien | Asien | 133 | 0,2 % | 7 | 127 | 133 | 0 | 0 | 33.892 |
| Maghreb | Afrika | 127 | 0,2 % | 10 | 117 | 127 | 0 | 0 | 33.707 |
| Ägypten | Afrika | 127 | 0,2 % | 12 | 116 | 127 | 0 | 0 | 33.737 |
| Bangladesch | Asien | 125 | 0,2 % | 5 | 120 | 125 | 0 | 0 | 33.862 |
| Pakistan | Asien | 123 | 0,2 % | 7 | 116 | 123 | 0 | 0 | 33.801 |
| Vietnam | Asien | 116 | 0,2 % | 6 | 109 | 116 | 0 | 0 | 33.883 |
| Brasilien | Südamerika | 115 | 0,2 % | 2 | 112 | 115 | 0 | 0 | 42.507 |
| Südliches Afrika | Afrika | 105 | 0,1 % | 6 | 99 | 105 | 0 | 0 | 33.905 |
| Iran und Irak | Asien | 102 | 0,1 % | 7 | 95 | 102 | 0 | 0 | 33.759 |
| Kolumbien, Venezuela und Ecuador | Südamerika | 96 | 0,1 % | 9 | 88 | 96 | 0 | 0 | 33.890 |
| Nigeria | Afrika | 96 | 0,1 % | 4 | 92 | 96 | 0 | 0 | 33.794 |
| Korea | Asien | 92 | 0,1 % | 5 | 87 | 92 | 0 | 0 | 33.879 |
| Äthiopien, Sudan und Horn von Afrika | Afrika | 73 | 0,1 % | 2 | 71 | 73 | 0 | 0 | 33.818 |
| Westafrika | Afrika | 71 | 0,1 % | 2 | 68 | 71 | 0 | 0 | 33.813 |
| Philippinen | Asien | 64 | 0,1 % | 4 | 60 | 64 | 0 | 0 | 33.918 |
| Levante und Israel | Asien | 56 | 0,1 % | 7 | 49 | 56 | 0 | 0 | 33.734 |
| Ostafrika | Afrika | 51 | 0,1 % | 1 | 50 | 51 | 0 | 0 | 33.860 |
| Kongo und Zentralafrika | Afrika | 37 | 0,1 % | 1 | 37 | 37 | 0 | 0 | 33.839 |
| Arabische Halbinsel | Asien | 33 | 0,0 % | 2 | 31 | 33 | 0 | 0 | 33.794 |
| Taiwan | Asien | 26 | 0,0 % | 2 | 25 | 26 | 0 | 0 | 33.895 |
| Malaysia, Singapur und Brunei | Asien | 21 | 0,0 % | 1 | 19 | 21 | 0 | 0 | 33.913 |

## Einordnung

- Reale Neuzulassungen weltweit: etwa 0,5 Mio. (1913), 4 Mio. (1929), 10 Mio. (1950), 30 Mio. (1970),
  50 Mio. (1990), 75 Mio. (2010), rund 80 Mio. Pkw (2025). Der Spielbedarf liegt deutlich darunter,
  vor allem nach 1950; Hauptgründe sind die Kaufschwelle 16 und die Preisempfindlichkeit 4 – zum Marktpreis
  kaufen nur die oberen Einkommensgruppen. Bei einer Abstimmung wären das die ersten Stellschrauben.
- Werte eines einzelnen Startmonats; im laufenden Spiel ändern Preise, Bestand (Besitzquote) und
  Ereignisse den Bedarf.
