"""Zusammenfassung der Länder zu Regionen (offene Punkte, Abschnitt I; M34).

Kleinstaaten gehen im Nachbarland auf, kleine Länder bilden Regionen. Eine Region behält
den ISO-Code ihres größten Mitglieds, wenn es mehr als die Hälfte der Einwohner stellt,
sonst bekommt sie einen frei verfügbaren Code (ISO 3166: X__).
"""

# Region → Mitglieder außer ihr selbst (bei Codes eines Mitglieds) bzw. alle Mitglieder.
# Seit dem Wunsch vom 11.10.2026 rund 50 Länder und Regionen (Welt von 2026).
REGIONEN = {
    # Europa
    "GBR": ["IRL"],
    "ITA": ["SMR", "MLT"],
    "ESP": ["PRT", "AND"],
    "FRA": ["MCO"],
    "NLD": ["BEL", "LUX"],
    "XAL": ["CHE", "LIE", "AUT"],
    "XNO": ["SWE", "NOR", "ISL", "DNK", "FIN", "EST", "LVA", "LTU"],
    "POL": ["CZE", "SVK", "HUN"],
    "UKR": ["BLR"],
    "XSO": ["ROU", "MDA", "BGR", "GRC", "CYP", "SRB", "HRV", "BIH", "MNE", "MKD", "ALB", "XKX",
            "SVN"],
    # Asien
    "IND": ["NPL", "BTN", "LKA", "MDV"],
    "KOR": ["PRK"],
    "IDN": ["TLS"],
    "MYS": ["BRN", "SGP"],
    "XSN": ["THA", "KHM", "LAO", "MMR"],
    "XAR": ["SAU", "YEM", "ARE", "KWT", "QAT", "BHR", "OMN"],
    "IRN": ["IRQ"],
    "XLE": ["ISR", "JOR", "LBN", "PSE", "SYR"],
    "TUR": ["AZE", "GEO", "ARM"],
    "XZA": ["KAZ", "UZB", "TJK", "KGZ", "TKM", "AFG", "MNG"],
    # Afrika
    "XMG": ["DZA", "MAR", "ESH", "TUN", "LBY"],
    "XSA": ["ZAF", "NAM", "BWA", "LSO", "SWZ", "ZWE", "ZMB", "MWI", "MOZ"],
    "XOA": ["KEN", "TZA", "UGA", "RWA", "BDI", "MDG", "MUS", "COM", "SYC"],
    "ETH": ["SOM", "DJI", "ERI", "SDN", "SSD"],
    "XWA": ["GHA", "CIV", "TGO", "BEN", "SEN", "GMB", "MLI", "BFA", "NER", "GIN", "SLE", "LBR",
            "GNB", "CPV", "MRT"],
    "COD": ["CMR", "AGO", "TCD", "CAF", "COG", "GAB", "GNQ", "STP"],
    # Amerika
    "XME": ["GTM", "HND", "NIC", "SLV", "CRI", "PAN", "BLZ", "CUB", "HTI", "DOM", "JAM", "TTO",
            "BHS", "BRB", "LCA", "GRD", "VCT", "ATG", "DMA", "KNA"],
    "ARG": ["PRY", "URY"],
    "COL": ["VEN", "ECU", "GUY", "SUR"],
    "PER": ["BOL", "CHL"],
    # Ozeanien
    "AUS": ["NZL", "PNG", "FJI", "SLB", "VUT", "WSM", "KIR", "FSM", "TON", "MHL", "PLW", "NRU",
            "TUV"],
}

# Deutsche Namen der Regionen (Länder mit ihrem eigenen Code, die Nachbarn aufnehmen,
# heißen nur dann anders, wenn das aufgenommene Land mehr als ein Kleinstaat ist).
NAMEN = {
    "GBR": "Vereinigtes Königreich und Irland",
    "ITA": "Italien und Malta",
    "ESP": "Spanien und Portugal",
    "NLD": "Benelux",
    "XAL": "Schweiz und Österreich",
    "XNO": "Nordeuropa und Baltikum",
    "POL": "Polen, Tschechien, Slowakei, Ungarn",
    "UKR": "Ukraine und Belarus",
    "XSO": "Südosteuropa",
    "IND": "Indien, Nepal und Sri Lanka",
    "KOR": "Korea",
    "IDN": "Indonesien und Osttimor",
    "MYS": "Malaysia, Singapur und Brunei",
    "XSN": "Festland-Südostasien",
    "XAR": "Arabische Halbinsel",
    "IRN": "Iran und Irak",
    "XLE": "Levante und Israel",
    "TUR": "Türkei und Kaukasus",
    "XZA": "Zentralasien und Mongolei",
    "XMG": "Maghreb",
    "XSA": "Südliches Afrika",
    "XOA": "Ostafrika",
    "ETH": "Äthiopien, Sudan und Horn von Afrika",
    "XWA": "Westafrika",
    "COD": "Kongo und Zentralafrika",
    "XME": "Mittelamerika und Karibik",
    "ARG": "Argentinien, Paraguay und Uruguay",
    "COL": "Kolumbien, Venezuela und Ecuador",
    "PER": "Peru, Bolivien und Chile",
    "AUS": "Australien und Ozeanien",
}

# Länder, deren Werte (Steuer, Stabilität, Prägung) für eine Region gelten, die nicht ihren
# Code trägt.
LEITLAND = {"XAL": "CHE", "XNO": "SWE", "XSO": "GRC", "XSA": "ZAF", "XMG": "DZA",
            "XSN": "THA", "XAR": "SAU", "XLE": "ISR", "XZA": "KAZ", "XOA": "KEN",
            "XWA": "GHA", "XME": "CUB"}

# Codes der Regionen vor dem 11.10.2026 (M34), die es nicht mehr gibt: Alte Spielstände
# lesen sie als die Region, in der ihre Länder jetzt liegen.
FRUEHER = {
    "XBA": "XNO", "XWB": "XSO", "XKA": "TUR", "XGO": "XAR", "XMA": "COD", "XTB": "XWA",
    "XRB": "XOA", "XHA": "ETH", "XHI": "XME", "XKB": "XME", "XPU": "ARG", "XGU": "COL",
    "XOZ": "AUS",
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
