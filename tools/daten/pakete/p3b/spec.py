# Spezifikation P3b – Luft- und Raumfahrt (Felder wie in p1/spec.py und p3a/spec.py).
# Preise zum Erfindungsjahr, USD Kaufkraft 2026. Staatsnachfrage über st(‰ des BIP, Preis):
# Fluggesellschaften, Flughäfen, Wetterdienste und Raumfahrtbehörden kaufen als Staat und
# Wirtschaft; Rüstungsgüter mit kf="ruestung" folgen den Militärausgaben (H3).

LM, TX, HB, CH, ME, MB, FARM = "lm", "tx", "hb", "ch", "me", "mb", "farm"
G, GB, LX = "grundbedarf", "gebrauchsgut", "luxus"


def P(**k): return k


def E(id, name, preis, einheit, gewicht, nach, rez, wg="fahrzeuge", branche="luftfahrt", tk="stueckgut", art="endprodukt", verw="konsum", **k):
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


RUESTUNG = "ruestung"  # demand follows military spending (H3); promille at the reference share
INDUSTRIE = dict(verw="industrie")
P3B = [
 # Werkstoffe und Bauteile
 P(id="titanerz", name="Titanerz (Ilmenit, Rutil)", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=300, verw="industrie",
   rez=dict(anlage=("titanerzgrube","Titanerzgrube","foerderstaette",3), h=10, profil=FARM, abbau=True, rname="Titanerz fördern"),
   index={1900:1.0,1950:1.0,1970:3.0,1990:5.0,2010:8.0,2026:9.0}, pacht=0.3,
   dep=[("titan_aus","Mineralsande Westaustralien","AUS",2e5,0.8,6e7),("titan_zaf","Richards Bay","ZAF",2e5,0.8,6e7),("titan_nor","Tellnes","NOR",1e5,0.9,3e7),
        ("titan_can","Lac Tio (Québec)","CAN",1.5e5,0.9,5e7),("titan_usa","Mineralsande Florida","USA",1e5,1.0,2e7),("titan_ind","Strandsande Kerala","IND",1e5,0.9,4e7),
        ("titan_chn","Panzhihua","CHN",1.5e5,1.0,8e7),("titan_ukr","Titanerz Ukraine","UKR",1e5,1.0,3e7),("titan_moz","Mineralsande Mosambik","MOZ",1e5,0.9,4e7)]),
 E("titan","Titan",30000,"t",None,None,R("titanwerk","Titanwerk",{"titanerz":3.0,"chlor":2.0,"kohle":1.0},60,profil=ME,tech="kroll_verfahren",umsatz=40,rname="Titan nach Kroll gewinnen"),
   # Implants, chemical plant and pigments besides aircraft.
   art="halbzeug", verw="industrie", wg="nichteisenmetalle", branche="metallurgie", staat=st(0.02,30000,2.0,{1950:0.1,1970:1.0,2026:2.0})),
 E("industriegase","Technische Gase (Sauerstoff, Stickstoff)",300,"t",None,None,R("luftzerlegungsanlage","Luftzerlegungsanlage",{"kohle":0.4},3,profil=CH,tech="luftzerlegung",umsatz=10,rname="Luft verflüssigen und zerlegen"),
   # Welding, steel works and hospitals buy most of it.
   art="halbzeug", verw="industrie", wg="chemie", branche="chemie", tk="fluessig", staat=st(0.2,300,1.5,{1900:0.1,1930:0.5,1960:1.0,2026:1.5})),
 E("kohlefaser","Kohlenstofffaser",60000,"t",None,None,R("kohlefaserwerk","Kohlenstofffaserwerk",{"polypropylen":1.5,"ammoniak":1.0},120,profil=CH,tech="kohlenstofffaser",umsatz=30,rname="Fasern spinnen und verkohlen"),
   art="halbzeug", verw="industrie", wg="chemie", branche="chemie",
   # Rackets, bicycles and wind blades besides aircraft.
   staat=st(0.005,60000,1.0,{1970:0.2,1990:1.0,2026:3.0})),
 E("avionik","Bordelektronik",60000,"stueck",40,None,R("avionikwerk","Avionikwerk",{"transistor":2000,"kupferdraht":0.02,"aluminium":0.05,"glas":0.01},250,tech="bordelektronik",umsatz=20,rname="Bordelektronik bauen"),
   art="komponente", verw="industrie", wg="elektro", branche="elektro"),
 E("wellenturbine","Propeller- und Wellenturbine",400000,"stueck",400,None,R("wellenturbinenwerk","Wellenturbinenwerk",{"stahl":0.4,"aluminium":0.3,"kupferdraht":0.02},1600,tech="propellerturbine",umsatz=40),
   art="komponente", verw="industrie"),
 E("mantelstromtriebwerk","Mantelstromtriebwerk",4000000,"stueck",4000,None,R("mantelstromwerk","Mantelstromtriebwerkswerk",{"titan":1.5,"aluminium":1.0,"stahl":1.0,"kupferdraht":0.05},16000,tech="mantelstromtriebwerk",umsatz=200),
   art="komponente", verw="industrie"),
 E("raketentriebwerk","Raketentriebwerk",3000000,"stueck",1500,None,R("raketentriebwerkswerk","Raketentriebwerkswerk",{"stahl":4.0,"kupfer":1.5,"aluminium":1.0},12000,tech="fluessigkeitsrakete",umsatz=60),
   art="komponente", verw="industrie"),
 E("satellitenbus","Satellitenplattform",20000000,"stueck",1000,None,R("satellitenbuswerk","Satellitenplattformwerk",{"avionik":4,"aluminium":1.0,"kupferdraht":0.2,"titan":0.2},70000,tech="satellitentechnik",umsatz=80),
   art="komponente", verw="industrie", komplex=True),
 # Kraftstoff
 E("kerosin","Kerosin (Flugkraftstoff)",900,"t",None,None,
   R("kerosinraffinerie","Kerosindestillation",{"rohoel":1.0},1.0,profil=CH,menge=0.1,umsatz=20,tech="strahltriebwerk",rname="Kerosin destillieren"),
   wg="mineraloel", branche="energie", tk="fluessig", verw="industrie",
   staat=st(0.3,900,2.0,{1939:0.05,1952:0.3,1970:1.5,1990:2.5,2026:3.0})),
 # Zivile Luftfahrt
 E("luftschiff","Starrluftschiff",1500000,"stueck",20000,None,
   R("luftschiffwerft","Luftschiffwerft",{"motor":4,"aluminium":5,"stoff":3,"industriegase":10},10000,tech="starrluftschiff",umsatz=20,tage=60),
   staat=st(0.003,1500000,3.0,{1908:0.3,1912:1.0,1930:1.0,1937:0.3,1945:0.05,2026:0.05}), **INDUSTRIE),
 E("wasserflugzeug","Wasserflugzeug",300000,"stueck",2500,None,
   R("wasserflugzeugwerk","Wasserflugzeugwerk",{"flugmotor":1,"aluminium":0.5,"schnittholz":0.5,"stoff":0.05},2000,tech="wasserflugzeug",umsatz=20),
   staat=st(0.003,300000,3.0,{1912:1.0,1950:0.5,2026:0.2}), **INDUSTRIE),
 E("sportflugzeug","Leichtflugzeug",150000,"stueck",700,("g",25,0.01,200000,1.5,2.0,LX),
   R("leichtflugzeugwerk","Leichtflugzeugwerk",{"flugmotor":1,"aluminium":0.4,"stahl":0.1,"gummi":0.005},1000,tech="leichtflugzeug",umsatz=20)),
 E("segelflugzeug","Segelflugzeug",40000,"stueck",250,("g",25,0.005,150000,1.5,2.0,LX),
   R("segelflugzeugbau","Segelflugzeugbau",{"schnittholz":0.2,"stoff":0.02,"leim":0.02},350,profil=HB,tech="segelflug",umsatz=5)),
 E("regionalflugzeug","Turboprop-Regionalflugzeug",6000000,"stueck",12000,None,
   R("turbopropwerk","Turbopropflugzeugwerk",{"wellenturbine":2,"aluminium":8,"avionik":1,"gummi":0.1},25000,tech="turbopropflugzeug",umsatz=150),
   staat=st(0.05,6000000,2.0,{1953:0.3,1970:1.0,2026:1.2}), **INDUSTRIE),
 E("frachtflugzeug","Frachtflugzeug",20000000,"stueck",40000,None,
   R("frachtflugzeugwerk","Frachtflugzeugwerk",{"wellenturbine":4,"aluminium":25,"avionik":2,"stahl":5},80000,tech="frachtflugzeug",umsatz=200),
   staat=st(0.05,20000000,3.0,{1955:0.3,1980:1.0,2026:2.0}), **INDUSTRIE),
 E("geschaeftsreiseflugzeug","Geschäftsreiseflugzeug",12000000,"stueck",10000,None,
   R("businessjetwerk","Geschäftsreisejetwerk",{"strahltriebwerk":2,"aluminium":4,"avionik":2,"glas":0.2},45000,tech="geschaeftsreisejet",umsatz=150),
   staat=st(0.03,12000000,1.0,{1963:0.3,1990:1.0,2026:1.5}), **INDUSTRIE),
 E("grossraumflugzeug","Großraumflugzeug",150000000,"stueck",150000,None,
   R("grossraumflugzeugwerk","Großraumflugzeugwerk",{"mantelstromtriebwerk":4,"aluminium":120,"avionik":8,"titan":10},600000,tech="grossraumflugzeug",umsatz=1500,tage=30),
   staat=st(0.3,150000000,2.0,{1970:0.3,1990:1.0,2010:2.0,2026:2.5}), **INDUSTRIE),
 E("regionaljet","Regionaljet",30000000,"stueck",22000,None,
   R("regionaljetwerk","Regionaljetwerk",{"mantelstromtriebwerk":2,"aluminium":15,"avionik":3,"titan":1},120000,tech="regionaljet",umsatz=300),
   staat=st(0.05,30000000,1.5,{1992:0.3,2005:1.0,2026:1.0}), **INDUSTRIE),
 E("hubschrauber","Hubschrauber",1500000,"stueck",1500,None,
   R("hubschrauberwerk","Hubschrauberwerk",{"flugmotor":1,"aluminium":1.0,"stahl":0.3},6000,tech="hubschrauber",umsatz=60),
   staat=st(0.05,1500000,3.0,{1944:0.2,1960:1.0,2026:1.5}), **INDUSTRIE),
 E("ultraleichtflugzeug","Ultraleichtflugzeug",60000,"stueck",300,("g",20,0.01,120000,1.5,2.0,LX),
   R("ultraleichtwerk","Ultraleichtflugzeugwerk",{"motor":1,"aluminium":0.1,"polyesterfaser":0.05,"stahl":0.05},400,tech="ultraleichtflugzeug",umsatz=10)),
 E("gleitschirm","Gleitschirm",3000,"stueck",6,("g",8,0.01,60000,1.5,2.0,LX),
   R("gleitschirmnaeherei","Gleitschirmnäherei",{"nylon":0.008,"polyesterfaser":0.002},40,profil=TX,tech="gleitschirm",umsatz=5)),
 E("heissluftballon","Heißluftballon",40000,"stueck",400,("g",15,0.003,150000,1.5,2.0,LX),
   R("ballonbau","Ballonbau",{"nylon":0.1,"stahl":0.05,"schnittholz":0.05},400,profil=TX,tech="heissluftballon",umsatz=5)),
 E("drachen","Drachen",20,"stueck",0.3,("g",3,0.1,8000,1.5,1.5,GB),
   R("drachenbau","Drachenbau",{"stoff":0.0003,"schnittholz":0.0005},0.25,profil=TX,umsatz=2,rname="Drachen bauen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("modellflugzeug","Modellflugzeug",150,"stueck",1.5,("g",4,0.03,25000,1.5,1.5,GB),
   R("modellbauwerk","Modellbauwerk",{"schnittholz":0.001,"kleinmotor":1,"leim":0.0005},1.0,tech="modellflug",umsatz=5),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("freizeitdrohne","Kameradrohne",900,"stueck",1.0,("g",3,0.08,30000,1.5,1.5,GB),
   R("drohnenwerk","Drohnenwerk",{"mikrochip":30,"lithium_ionen_akku":6,"kleinmotor":4,"kunststoffwaren":0.5},4,tech="multikopter",umsatz=40),
   wg="elektro", branche="elektro"),
 E("industriedrohne","Vermessungs- und Lieferdrohne",25000,"stueck",20,None,
   R("industriedrohnenwerk","Industriedrohnenwerk",{"mikrochip":200,"traktionsbatterie":2,"kleinmotor":8,"kohlefaser":0.005},100,tech="multikopter",umsatz=30),
   wg="elektro", branche="elektro", staat=st(0.01,25000,2.0,{2014:0.2,2020:1.0,2026:2.0}), **INDUSTRIE),
 E("fallschirm","Fallschirm",2000,"stueck",10,None,
   R("fallschirmnaeherei","Fallschirmnäherei",{"stoff":0.012},25,profil=TX,tech="fallschirm",umsatz=5),
   wg="ruestung", branche="ruestung", staat=st(0.01,2000,RUESTUNG), **INDUSTRIE),
 E("wetterballon","Radiosonde mit Wetterballon",300,"stueck",1.0,None,
   R("radiosondenwerk","Radiosondenwerk",{"gummi":0.0015,"radioroehre":3,"industriegase":0.002},1.5,tech="radiosonde",umsatz=5),
   wg="elektro", branche="elektro", staat=st(0.002,300,1.0,{1930:0.3,1960:1.0,2026:1.0}), **INDUSTRIE),
 E("flugsimulator","Flugsimulator",10000000,"stueck",15000,None,
   R("simulatorwerk","Simulatorwerk",{"mikrochip":20000,"aluminium":5,"elektromotor":6,"stahl":10},35000,tech="flugsimulator",umsatz=60),
   wg="elektro", branche="elektro", staat=st(0.01,10000000,1.0,{1965:0.3,1990:1.0,2026:1.0}), **INDUSTRIE),
 E("fluggastbruecke","Fluggastbrücke",800000,"stueck",30000,None,
   R("fluggastbrueckenwerk","Fluggastbrückenwerk",{"stahl":20,"glas":1.0,"elektromotor":4,"gummi":0.05},3000,tech="fluggastbruecke",umsatz=30),
   wg="maschinen", branche="maschinenbau", staat=st(0.005,800000,0.5,{1959:0.3,1980:1.0,2026:1.3}), **INDUSTRIE),
 E("flugsicherungsradar","Flugsicherungsradar",5000000,"stueck",20000,None,
   R("radarwerk","Radarwerk",{"radioroehre":2000,"kupferdraht":1.0,"stahl":10,"elektromotor":2},18000,tech="radar",umsatz=40),
   wg="elektro", branche="elektro", staat=st(0.01,5000000,2.0,{1935:0.2,1950:1.0,2026:1.0}), **INDUSTRIE),
 # Militärische Luftfahrt (Rüstungsgüter, H3)
 E("bomber","Bomber",3000000,"stueck",15000,None,
   R("bomberwerk","Bomberwerk",{"flugmotor":4,"aluminium":10,"stahl":2,"gummi":0.2},12000,tech="bomber",umsatz=150),
   wg="ruestung", staat=st(0.5,3000000,RUESTUNG), **INDUSTRIE),
 E("militaertransporter","Militärtransportflugzeug",30000000,"stueck",35000,None,
   R("militaertransporterwerk","Militärtransporterwerk",{"wellenturbine":4,"aluminium":30,"avionik":3,"stahl":5},110000,tech="militaertransporter",umsatz=300),
   wg="ruestung", staat=st(0.3,30000000,RUESTUNG), **INDUSTRIE),
 E("kampfhubschrauber","Kampfhubschrauber",20000000,"stueck",5000,None,
   R("kampfhubschrauberwerk","Kampfhubschrauberwerk",{"wellenturbine":2,"aluminium":3,"avionik":4,"titan":0.5},75000,tech="kampfhubschrauber",umsatz=200),
   wg="ruestung", staat=st(0.3,20000000,RUESTUNG), **INDUSTRIE),
 E("militaerdrohne","Militärdrohne",15000000,"stueck",1100,None,
   R("militaerdrohnenwerk","Militärdrohnenwerk",{"mikrochip":20000,"wellenturbine":1,"kohlefaser":1.0,"avionik":2},55000,tech="militaerdrohne",umsatz=150),
   wg="ruestung", staat=st(0.2,15000000,RUESTUNG), **INDUSTRIE),
 E("marschflugkoerper","Marschflugkörper",1500000,"stueck",1300,None,
   R("marschflugkoerperwerk","Marschflugkörperwerk",{"strahltriebwerk":1,"aluminium":0.4,"avionik":1,"stahl":0.3},5000,tech="marschflugkoerper",umsatz=100),
   wg="ruestung", branche="ruestung", staat=st(0.2,1500000,RUESTUNG), **INDUSTRIE),
 # Raumfahrt
 E("hoehenforschungsrakete","Höhenforschungsrakete",500000,"stueck",1500,None,
   R("forschungsraketenwerk","Forschungsraketenwerk",{"stahl":1.0,"aluminium":0.5,"industriegase":2,"radioroehre":50},2000,tech="hoehenforschungsrakete",umsatz=20),
   branche="raumfahrt", staat=st(0.005,500000,1.0,{1946:0.3,1960:1.0,2026:0.5}), **INDUSTRIE),
 E("traegerrakete","Trägerrakete",60000000,"stueck",300000,None,
   R("raketenwerk","Raketenwerk",{"raketentriebwerk":6,"aluminium":40,"industriegase":500,"avionik":2},220000,tech="traegerrakete",umsatz=500,tage=30),
   branche="raumfahrt", staat=st(0.08,60000000,1.0,{1957:0.2,1965:1.0,1990:0.7,2026:1.0}), **INDUSTRIE),
 E("wiederverwendbare_rakete","Wiederverwendbare Trägerrakete",50000000,"stueck",400000,None,
   R("landeraketenwerk","Werk für wiederverwendbare Raketen",{"raketentriebwerk":9,"aluminium":30,"kohlefaser":5,"avionik":4},180000,tech="wiederverwendbare_rakete",umsatz=500,tage=30),
   branche="raumfahrt", staat=st(0.05,50000000,1.0,{2015:0.3,2020:1.0,2026:2.0}), **INDUSTRIE),
 E("kommunikationssatellit","Kommunikationssatellit",150000000,"stueck",3000,None,
   R("satellitenwerk","Satellitenwerk",{"satellitenbus":1,"avionik":6,"kupferdraht":0.5,"aluminium":1.0},550000,tech="kommunikationssatellit",umsatz=800,tage=30),
   branche="raumfahrt", staat=st(0.05,150000000,1.0,{1965:0.2,1985:1.0,2026:1.5}), **INDUSTRIE),
 E("wettersatellit","Wettersatellit",100000000,"stueck",2000,None,
   R("wettersatellitenwerk","Wettersatellitenwerk",{"satellitenbus":1,"avionik":3,"glas":0.2,"aluminium":0.5},370000,tech="wettersatellit",umsatz=500,tage=30),
   branche="raumfahrt", staat=st(0.03,100000000,1.0,{1960:0.3,1980:1.0,2026:1.0}), **INDUSTRIE),
 E("navigationssatellit","Navigationssatellit",150000000,"stueck",2000,None,
   R("navigationssatellitenwerk","Navigationssatellitenwerk",{"satellitenbus":1,"avionik":8,"kupferdraht":0.3,"aluminium":0.5},550000,tech="satellitennavigation",umsatz=800,tage=30),
   branche="raumfahrt", staat=st(0.03,150000000,1.0,{1978:0.3,1995:1.0,2026:1.0}), **INDUSTRIE),
 E("erdbeobachtungssatellit","Erdbeobachtungssatellit",120000000,"stueck",2000,None,
   R("erdbeobachtungswerk","Erdbeobachtungssatellitenwerk",{"satellitenbus":1,"avionik":4,"glas":0.5,"aluminium":0.5},440000,tech="erdbeobachtung",umsatz=600,tage=30),
   branche="raumfahrt", staat=st(0.03,120000000,1.0,{1972:0.3,1990:1.0,2026:1.5}), **INDUSTRIE),
 E("raumsonde","Raumsonde",400000000,"stueck",1500,None,
   R("raumsondenwerk","Raumsondenwerk",{"satellitenbus":1,"avionik":10,"titan":0.3,"kupferdraht":0.2},1500000,tech="raumsonde",umsatz=1000,tage=60),
   branche="raumfahrt", staat=st(0.02,400000000,1.0,{1962:0.3,1975:1.0,2026:1.0}), **INDUSTRIE),
 E("kleinsatellit","Kleinsatellit",1000000,"stueck",10,None,
   R("kleinsatellitenwerk","Kleinsatellitenwerk",{"mikrochip":2000,"lithium_ionen_akku":40,"aluminium":0.01,"kupferdraht":0.001},3500,tech="kleinsatellit",umsatz=50),
   branche="raumfahrt", staat=st(0.01,1000000,1.0,{2003:0.1,2015:1.0,2026:3.0}), **INDUSTRIE),
 E("satellitenempfaenger","Satellitenempfänger",400,"stueck",6,("g",8,0.3,20000,1.0,1.0,GB),
   R("satellitenempfaengerwerk","Satellitenempfängerwerk",{"mikrochip":20,"aluminium":0.004,"kunststoffwaren":1,"kupferdraht":0.0005},1.5,tech="satellitenfernsehen",umsatz=40),
   wg="elektro", branche="elektro"),
 E("satellitentelefon","Satellitentelefon",1500,"stueck",0.3,("g",5,0.01,80000,1.5,1.5,LX),
   R("satellitentelefonwerk","Satellitentelefonwerk",{"mikrochip":50,"lithium_ionen_akku":3,"kunststoffwaren":0.2,"lcd_panel":0.003},5,tech="satellitentelefonie",umsatz=20),
   wg="elektro", branche="elektro"),
 E("navigationsgeraet","Navigationsgerät",400,"stueck",0.3,("g",5,0.3,25000,1.2,1.2,GB),
   R("navigationsgeraetewerk","Navigationsgerätewerk",{"mikrochip":40,"lcd_panel":0.01,"lithium_ionen_akku":2,"kunststoffwaren":0.3},1.5,tech="navigationsgeraet",umsatz=40),
   wg="elektro", branche="elektro"),
]

for _p in P3B:
    # Aircraft and spacecraft built from engines, electronics and airframes: five or six
    # levels from the ore (Lastenheft §17.2).
    if _p.get("art") == "endprodukt" and _p["id"] not in ("kerosin", "segelflugzeug", "gleitschirm", "heissluftballon", "drachen", "fallschirm", "modellflugzeug", "wetterballon"):
        _p["komplex"] = True
