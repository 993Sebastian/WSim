"""Zusammenfassung der Länder zu Regionen (offene Punkte, Abschnitt I; M34).

Kleinstaaten gehen im Nachbarland auf, kleine Länder bilden Regionen. Eine Region behält
den ISO-Code ihres größten Mitglieds, wenn es mehr als die Hälfte der Einwohner stellt,
sonst bekommt sie einen frei verfügbaren Code (ISO 3166: X__).
"""

# Region → Mitglieder außer ihr selbst (bei Codes eines Mitglieds) bzw. alle Mitglieder.
REGIONEN = {
    # Europa
    "ESP": ["AND"],
    "FRA": ["MCO"],
    "ITA": ["SMR", "MLT"],
    "CHE": ["LIE"],
    "BEL": ["LUX"],
    "NOR": ["ISL"],
    "ROU": ["MDA"],
    "CZE": ["SVK"],
    "GRC": ["CYP"],
    "XBA": ["EST", "LVA", "LTU"],
    "XWB": ["SRB", "HRV", "BIH", "MNE", "MKD", "ALB", "XKX", "SVN"],
    # Asien
    "IDN": ["TLS"],
    "MYS": ["BRN"],
    "NPL": ["BTN"],
    "LKA": ["MDV"],
    "KHM": ["LAO"],
    "XKA": ["AZE", "GEO", "ARM"],
    "XZA": ["TJK", "KGZ", "TKM"],
    "XGO": ["ARE", "KWT", "QAT", "BHR", "OMN"],
    "XLE": ["JOR", "LBN", "PSE"],
    # Afrika
    "MAR": ["ESH"],
    "SDN": ["SSD"],
    "SEN": ["GMB"],
    "MDG": ["MUS", "COM", "SYC"],
    "XMA": ["TCD", "CAF", "COG", "GAB", "GNQ", "STP"],
    "XWA": ["GIN", "SLE", "LBR", "GNB", "CPV", "MRT"],
    "XTB": ["TGO", "BEN"],
    "XRB": ["RWA", "BDI"],
    "XHA": ["SOM", "DJI", "ERI"],
    "XSA": ["NAM", "BWA", "LSO", "SWZ"],
    # Amerika
    "XHI": ["HTI", "DOM"],
    "XME": ["GTM", "HND", "NIC", "SLV", "CRI", "PAN", "BLZ"],
    "XKB": ["JAM", "TTO", "BHS", "BRB", "LCA", "GRD", "VCT", "ATG", "DMA", "KNA"],
    "XPU": ["PRY", "URY"],
    "XGU": ["GUY", "SUR"],
    # Ozeanien
    "XOZ": ["PNG", "FJI", "SLB", "VUT", "WSM", "KIR", "FSM", "TON", "MHL", "PLW", "NRU", "TUV"],
}

# Deutsche Namen der Regionen (Länder mit ihrem eigenen Code, die Nachbarn aufnehmen,
# heißen nur dann anders, wenn das aufgenommene Land mehr als ein Kleinstaat ist).
NAMEN = {
    "BEL": "Belgien und Luxemburg",
    "NOR": "Norwegen und Island",
    "ROU": "Rumänien und Moldau",
    "CZE": "Tschechien und Slowakei",
    "GRC": "Griechenland und Zypern",
    "ITA": "Italien und Malta",
    "IDN": "Indonesien und Osttimor",
    "MYS": "Malaysia und Brunei",
    "NPL": "Nepal und Bhutan",
    "LKA": "Sri Lanka und Malediven",
    "KHM": "Kambodscha und Laos",
    "MAR": "Marokko und Westsahara",
    "SDN": "Sudan und Südsudan",
    "SEN": "Senegal und Gambia",
    "MDG": "Madagaskar und Inseln",
    "XBA": "Baltikum",
    "XWB": "Westbalkan",
    "XKA": "Kaukasus",
    "XZA": "Zentralasien",
    "XGO": "Golfstaaten",
    "XLE": "Levante",
    "XMA": "Mittelafrika",
    "XWA": "Westafrika",
    "XTB": "Togo und Benin",
    "XRB": "Ruanda und Burundi",
    "XHA": "Horn von Afrika",
    "XSA": "Südliches Afrika",
    "XHI": "Hispaniola",
    "XME": "Mittelamerika",
    "XKB": "Karibik",
    "XPU": "Paraguay und Uruguay",
    "XGU": "Guayana",
    "XOZ": "Ozeanien",
}


def region_of():
    """Land → Region für alle zusammengefassten Länder (Länder ohne Eintrag bleiben)."""
    mapping = {}
    for region, members in REGIONEN.items():
        for m in members:
            assert m not in mapping, m
            mapping[m] = region
    return mapping


def members(region):
    """Alle Mitglieder einer Region einschließlich des namensgebenden Landes."""
    own = [] if region.startswith("X") else [region]
    return own + REGIONEN.get(region, [])
