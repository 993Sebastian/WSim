# Spezifikation P3a – Fahrzeuge, Bahn und Schiffe (Felder wie in p1/spec.py; nach: zusätzlich
# ('e', zu, je_besitz_und_jahr, X, pe, ie, klasse) für Ergänzungsgüter). Preise um 1900 bzw.
# zum Erfindungsjahr, USD Kaufkraft 2026. Staatsnachfrage über st(‰ des BIP, Preis): Bahn,
# Busse, Schiffe und Lieferwagen kaufen Staat und Wirtschaft (wie Maschinen in P2).

LM, TX, HB, CH, ME, MB, FARM = "lm", "tx", "hb", "ch", "me", "mb", "farm"
G, GB, LX = "grundbedarf", "gebrauchsgut", "luxus"


def P(**k): return k


def E(id, name, preis, einheit, gewicht, nach, rez, wg="fahrzeuge", branche="fahrzeuge", tk="stueckgut", art="endprodukt", verw="konsum", **k):
    d = dict(id=id, name=name, art=art, branche=branche, einheit=einheit, wg=wg, tk=tk, preis=preis, verw=verw, nach=nach, rez=rez)
    if gewicht is not None: d["gewicht"] = gewicht
    d.update(k)
    return d


def R(anlage, aname, ein, h, profil=MB, tech=None, menge=1, umsatz=5, typ="werk", rname=None, neben=None, tage=1):
    r = dict(anlage=(anlage, aname, typ, umsatz), ein=ein, h=h, profil=profil, menge=menge, tage=tage)
    if tech: r["tech"] = tech
    if rname: r["rname"] = rname
    if neben: r["neben"] = neben
    return r


def st(promille, preis, kf=1.0, verlauf=None):
    """State demand worth `promille` ‰ of GDP a year: units per million USD of GDP."""
    je = float(f"{promille * 1000 / preis:.3g}")
    return (je, kf, verlauf) if verlauf else (je, kf)


INDUSTRIE = dict(verw="industrie")
P3A = [
 # Blei und Batterien
 P(id="bleierz", name="Bleierz", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=150, verw="industrie",
   rez=dict(anlage=("bleigrube","Bleigrube","foerderstaette",3), h=14, profil=FARM, abbau=True, rname="Bleierz fördern"),
   index={1900:1.0,1930:1.6,1950:1.8,1970:3.0,1990:3.2,2010:4.5,2026:5.0}, pacht=0.3,
   dep=[("blei_aus","Broken Hill (Blei)","AUS",4e5,0.8,6e7),("blei_usa","Bleigürtel Missouri","USA",6e5,0.9,8e7),("blei_mex","Bleigruben Mexiko","MEX",2.5e5,0.9,4e7),
        ("blei_esp","Linares","ESP",3e5,1.0,2e7),("blei_deu","Harz und Oberharz","DEU",2e5,1.1,1e7),("blei_per","Cerro de Pasco","PER",1.5e5,0.9,4e7),
        ("blei_chn","Bleigruben Hunan","CHN",1.5e5,1.0,8e7),("blei_rus","Blei Altai","KAZ",1e5,1.0,4e7),("blei_can","Blei British Columbia","CAN",1e5,0.9,3e7)]),
 E("blei","Blei",2500,"t",None,None,R("bleihuette","Bleihütte",{"bleierz":1.6,"kohle":0.5},6,profil=ME,rname="Bleierz rösten und schmelzen"),
   # Lead pipes, cable sheaths, paint and ammunition: bought by state and industry.
   art="halbzeug", verw="industrie", wg="nichteisenmetalle", branche="metallurgie", staat=st(0.1, 2500, 2.0)),
 E("starterbatterie","Starterbatterie",120,"stueck",15,("e","automobil",0.25,6000,0.4,0.5,GB),
   R("bleiakkuwerk","Bleiakkuwerk",{"blei":0.009,"schwefelsaeure":0.004},1.2,profil=CH,tech="elektrischer_anlasser",rname="Bleiakkus gießen und füllen"), wg="elektro", branche="elektro"),
 E("dieselkraftstoff","Dieselkraftstoff",900,"t",None,None,
   R("dieselraffinerie","Mitteldestillation",{"rohoel":1.0},1.0,profil=CH,menge=0.35,umsatz=30,rname="Gasöl destillieren"),
   wg="mineraloel", branche="energie", tk="fluessig", verw="industrie",
   staat=st(0.3,900,2.0,{1900:0.05,1920:0.3,1940:1.0,1960:2.5,1980:4.0,2000:5.0,2026:4.5})),
 # Straßenfahrzeuge
 E("motorrad","Motorrad",4000,"stueck",150,("g",10,0.15,12000,1.0,1.5,GB),
   R("motorradwerk","Motorradwerk",{"stahl":0.06,"blech":0.01,"fahrradreifen":2},40),
   staat=st(0.01,4000,3.0)),
 E("motorroller","Motorroller",2500,"stueck",110,("g",8,0.2,5000,1.0,1.5,GB),
   R("rollerwerk","Rollerwerk",{"stahl":0.03,"blech":0.03,"fahrradreifen":2},25,tech="motorroller")),
 E("kleinwagen","Kleinwagen",10000,"stueck",750,("g",10,0.3,12000,1.5,1.5,GB),
   R("kleinwagenwerk","Kleinwagenwerk",{"motor":1,"fahrgestell":1,"karosserie":1,"autoreifen":4},40,tech="kleinwagen",umsatz=50)),
 E("sportwagen","Sportwagen",80000,"stueck",1200,("g",8,0.02,80000,1.5,2.0,LX),
   R("sportwagenmanufaktur","Sportwagenmanufaktur",{"motor":2,"fahrgestell":1,"karosserie":1,"autoreifen":4},600,tech="sportwagen",umsatz=20)),
 E("suv","Geländelimousine (SUV)",40000,"stueck",2000,("g",12,0.15,30000,1.5,1.5,GB),
   R("suvwerk","SUV-Werk",{"motor":1,"fahrgestell":1,"karosserie":1,"stahl":0.3},120,tech="suv",umsatz=60)),
 E("hybridantrieb","Hybridantrieb",5000,"stueck",150,None,
   R("hybridwerk","Hybridantriebswerk",{"motor":1,"elektroantrieb":1,"lithium_ionen_akku":120},20,tech="hybridantrieb",umsatz=40),
   art="komponente", verw="industrie"),
 E("hybridauto","Hybridauto",35000,"stueck",1450,("g",12,0.2,25000,1.5,1.5,GB),
   R("hybridautowerk","Hybridautowerk",{"hybridantrieb":1,"fahrgestell":1,"karosserie":1,"autoreifen":4},90,tech="hybridantrieb",umsatz=60), komplex=True),
 E("lieferwagen","Lieferwagen",15000,"stueck",1500,None,
   R("lieferwagenwerk","Lieferwagenwerk",{"motor":1,"fahrgestell":1,"karosserie":1,"autoreifen":4},60,tech="lieferwagen",umsatz=40),
   staat=st(0.3,15000,3.0,{1925:1.0,1950:2.0,1970:3.0,2026:4.0}), **INDUSTRIE),
 E("omnibus","Omnibus",50000,"stueck",9000,None,
   R("omnibuswerk","Omnibuswerk",{"dieselmotor":1,"fahrgestell":1,"blech":1.0,"autoreifen":6},250,tech="omnibus",umsatz=40),
   staat=st(0.06,50000,1.0,{1919:1.0,1950:2.0,2026:2.5}), **INDUSTRIE),
 E("wohnmobil","Wohnmobil",45000,"stueck",3000,("g",15,0.03,60000,1.5,2.0,LX),
   R("wohnmobilwerk","Wohnmobilwerk",{"motor":1,"fahrgestell":1,"karosserie":1,"schnittholz":0.4},200,tech="wohnmobil",umsatz=20)),
 # Bahn
 E("dampflokomotive","Dampflokomotive",450000,"stueck",80000,None,
   R("lokomotivfabrik","Lokomotivfabrik",{"stahl":70,"industriekessel":1,"kupfer":2},5000,umsatz=60,rname="Dampflokomotiven bauen"),
   staat=st(0.06,450000,1.5), **INDUSTRIE),
 E("diesellokomotive","Diesellokomotive",1500000,"stueck",100000,None,
   R("diesellokwerk","Diesellokwerk",{"stahl":70,"dieselmotor":4,"elektromotor":4},14000,tech="diesellokomotive",umsatz=80),
   staat=st(0.06,1500000,1.5), ersetzt=["dampflokomotive"], **INDUSTRIE),
 E("elektrolokomotive","Elektrolokomotive",2000000,"stueck",90000,None,
   R("elektrolokwerk","Elektrolokwerk",{"stahl":60,"elektromotor":6,"kupfer":4},18000,tech="elektrolokomotive",umsatz=80),
   staat=st(0.03,2000000,1.5,{1905:0.3,1930:1.0,1960:2.0,2026:3.0}), **INDUSTRIE),
 E("gueterwagen","Güterwagen",30000,"stueck",22000,None,
   R("waggonfabrik","Waggonfabrik",{"stahl":12,"stabstahl":2},250,umsatz=40,rname="Güterwagen bauen"),
   staat=st(0.15,30000,2.0), **INDUSTRIE),
 E("reisezugwagen","Reisezugwagen",150000,"stueck",40000,None,
   R("personenwagenfabrik","Personenwagenfabrik",{"stahl":20,"glas":0.5,"schnittholz":2},1300,umsatz=40,rname="Reisezugwagen bauen"),
   staat=st(0.06,150000,1.0), **INDUSTRIE),
 E("strassenbahn","Straßenbahn",150000,"stueck",20000,None,
   R("strassenbahnwerk","Straßenbahnwerk",{"stahl":12,"elektromotor":2,"glas":0.4},1300,umsatz=40,rname="Straßenbahnen bauen"),
   staat=st(0.03,150000,0.5), **INDUSTRIE),
 E("hochgeschwindigkeitszug","Hochgeschwindigkeitszug",30000000,"stueck",400000,None,
   R("schnellzugwerk","Hochgeschwindigkeitszugwerk",{"stahl":250,"elektromotor":40,"kupfer":20,"glas":5},250000,tech="hochgeschwindigkeitszug",umsatz=300),
   staat=st(0.02,30000000,0.5,{1964:0.2,1990:1.0,2010:2.5,2026:3.0}), **INDUSTRIE),
 # Schiffe (Schiffbau)
 E("frachtschiff","Dampffrachter",4000000,"stueck",3000000,None,
   R("werft","Werft",{"stahl":3000,"industriekessel":2,"kupfer":10},30000,umsatz=150,tage=30,rname="Frachtdampfer bauen"),
   branche="schiffbau", staat=st(0.1,4000000,3.0), **INDUSTRIE),
 E("motorschiff","Motorfrachter",8000000,"stueck",4000000,None,
   R("motorschiffswerft","Motorschiffswerft",{"stahl":4000,"dieselmotor":6},60000,tech="motorschiff",umsatz=200),
   branche="schiffbau", staat=st(0.1,8000000,3.0), ersetzt=["frachtschiff"], **INDUSTRIE),
 E("tankschiff","Tankschiff",10000000,"stueck",8000000,None,
   R("tankerwerft","Tankerwerft",{"stahl":8000,"dieselmotor":4,"pumpe":20},70000,tech="motorschiff",umsatz=200),
   branche="schiffbau", staat=st(0.05,10000000,2.0,{1912:0.5,1950:1.0,1975:3.0,2026:3.0}), **INDUSTRIE),
 E("containerschiff","Containerschiff",60000000,"stueck",30000000,None,
   R("containerwerft","Containerwerft",{"stahl":25000,"dieselmotor":2,"kran":4},400000,tech="containerschiff",umsatz=500),
   branche="schiffbau", staat=st(0.05,60000000,1.0,{1956:0.2,1970:1.0,2000:3.0,2026:4.0}), **INDUSTRIE),
 E("kreuzfahrtschiff","Kreuzfahrtschiff",300000000,"stueck",60000000,None,
   R("kreuzfahrtwerft","Kreuzfahrtwerft",{"stahl":40000,"dieselmotor":12,"schnittholz":2000},2500000,tech="kreuzfahrtschiff",umsatz=1500),
   branche="schiffbau", staat=st(0.01,300000000,0.5,{1966:0.2,1990:1.0,2010:2.5,2026:3.0}), **INDUSTRIE),
 E("sportboot","Sportboot",60000,"stueck",3000,("g",20,0.02,80000,1.5,2.0,LX),
   R("bootswerft","Bootswerft",{"polyesterfaser":0.6,"dieselmotor":1},400,tech="sportboot",umsatz=20),
   branche="schiffbau"),
]

for _p in P3A:
    # Ships, rail vehicles and motor vehicles are built from engines and bodies: five
    # levels from the ore (Lastenheft §17.2).
    if _p.get("art") == "endprodukt" and _p["id"] not in ("starterbatterie", "dieselkraftstoff", "motorrad", "motorroller", "gueterwagen", "reisezugwagen"):
        _p["komplex"] = True
