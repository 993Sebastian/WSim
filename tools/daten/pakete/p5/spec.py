# Spezifikation P5 – Energie- und Umwelttechnik, Medizintechnik, Freizeit (Felder wie in
# p1/spec.py und p3a/spec.py). Preise zum Erfindungsjahr, USD Kaufkraft 2026. Staatsnachfrage über
# st(‰ des BIP, Preis): Versorger, Kommunen und das Gesundheitswesen kaufen Kraftwerks-,
# Umwelt- und Medizintechnik als Staat und Wirtschaft.

LM, TX, HB, CH, ME, MB, EL, FARM = "lm", "tx", "hb", "ch", "me", "mb", "el", "farm"
G, GB, LX = "grundbedarf", "gebrauchsgut", "luxus"


def P(**k): return k


def E(id, name, preis, einheit, gewicht, nach, rez, wg="elektro", branche="elektro", tk="stueckgut", art="endprodukt", verw="konsum", **k):
    d = dict(id=id, name=name, art=art, branche=branche, einheit=einheit, wg=wg, tk=tk, preis=preis, verw=verw, nach=nach, rez=rez)
    if gewicht is not None: d["gewicht"] = gewicht
    d.update(k)
    return d


def R(anlage, aname, ein, h, profil=EL, tech=None, menge=1, umsatz=5, typ="werk", rname=None, neben=None, tage=1):
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
P5 = [
 # Rohstoff
 P(id="uranerz", name="Uranerz (Konzentrat)", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=2000, verw="industrie",
   rez=dict(anlage=("urangrube","Urangrube","foerderstaette",3), h=30, profil=FARM, abbau=True, rname="Uranerz fördern"),
   index={1900:1.0,1950:1.0,1970:3.0,1990:3.0,2010:4.0,2026:4.5}, pacht=0.3,
   dep=[("uran_can","Athabasca-Becken","CAN",4e3,0.8,4e5),("uran_aus","Olympic Dam und Ranger","AUS",3e3,0.8,5e5),("uran_kaz","Uran Kasachstan","KAZ",3e3,0.8,4e5),
        ("uran_ner","Arlit","NER",2e3,0.9,2e5),("uran_usa","Colorado-Plateau","USA",3e3,1.0,2e5),("uran_zaf","Witwatersrand (Uran)","ZAF",2e3,1.0,2e5),
        ("uran_cze","Joachimsthal","CZE",1e3,1.1,5e4),("uran_uzb","Uran Usbekistan","UZB",1e3,0.9,1e5),("uran_cod","Shinkolobwe","COD",1e3,1.0,5e4)]),
 # Energie- und Umwelttechnik
 E("wasserturbine","Wasserkraftturbine",500000,"stueck",40000,None,
   R("turbinenwerk","Turbinenwerk",{"stahl":30,"kupfer":1,"stabstahl":5},2500,profil=MB,tech="wasserkraftturbine",umsatz=40),
   wg="maschinen", branche="maschinenbau", staat=st(0.03,500000,1.0,{1905:0.5,1950:1.0,2026:0.7}), **INDUSTRIE),
 E("dampfturbine","Dampfturbine",3000000,"stueck",150000,None,
   R("dampfturbinenwerk","Dampfturbinenwerk",{"stahl":100,"industriekessel":2,"kupfer":2},15000,profil=MB,tech="turbogenerator",umsatz=150,tage=30),
   wg="maschinen", branche="maschinenbau", staat=st(0.05,3000000,1.0,{1903:0.3,1950:1.0,2026:0.6}), **INDUSTRIE),
 E("gasturbine","Gasturbine",15000000,"stueck",200000,None,
   R("gasturbinenwerk","Gasturbinenwerk",{"stahl":60,"nickel":5,"kupfer":2},60000,profil=MB,tech="gasturbinenkraftwerk",umsatz=300,tage=30),
   wg="maschinen", branche="maschinenbau", staat=st(0.03,15000000,1.0,{1939:0.1,1970:1.0,2026:1.5}), **INDUSTRIE),
 E("brennelement","Brennelement",1500000,"stueck",600,None,
   R("brennelementwerk","Brennelementwerk",{"uranerz":60,"stahl":0.2,"titan":0.05},4000,profil=CH,tech="kernkraftwerk",umsatz=200),
   wg="mineraloel", branche="energie", staat=st(0.03,1500000,1.0,{1956:0.1,1980:1.0,2000:0.9,2026:0.7}), **INDUSTRIE),
 E("solarkollektor","Solarkollektor",800,"stueck",40,("g",20,0.15,25000,1.2,1.2,GB),
   R("kollektorwerk","Kollektorwerk",{"glas":0.02,"kupfer":0.004,"aluminium":0.005},4,profil=ME,tech="solarthermie",umsatz=20)),
 E("wechselrichter","Solar-Wechselrichter",1000,"stueck",20,("g",15,0.2,30000,1.2,1.2,GB),
   R("wechselrichterwerk","Wechselrichterwerk",{"mikrochip":50,"kupferdraht":0.003,"aluminium":0.003,"leiterplatte":2},4,tech="netzwechselrichter",umsatz=40)),
 E("heimspeicher","Batteriespeicher für Haushalte",6000,"stueck",100,("g",15,0.1,40000,1.3,1.3,GB),
   R("heimspeicherwerk","Heimspeicherwerk",{"traktionsbatterie":10,"mikrochip":50,"stahl":0.02,"leiterplatte":2},20,tech="heimspeicher",umsatz=60)),
 E("wallbox","Ladestation (Wallbox)",800,"stueck",6,("e","elektroauto",0.1,25000,1.0,1.0,GB),
   R("wallboxwerk","Wallboxwerk",{"kupferdraht":0.002,"mikrochip":20,"kunststoffwaren":2,"kabel":0.003},3,tech="ladestation",umsatz=30)),
 E("elektrolyseur","Elektrolyseur (Wasserstoff)",2000000,"stueck",20000,None,
   R("elektrolyseurwerk","Elektrolyseurwerk",{"nickel":2,"stahl":10,"kupfer":2,"mikrochip":500},8000,tech="pem_elektrolyse",umsatz=100),
   wg="maschinen", branche="maschinenbau", staat=st(0.01,2000000,1.0,{2018:0.2,2026:1.0}), **INDUSTRIE),
 E("biogasanlage","Biogasanlage",1000000,"stueck",100000,None,
   R("biogaswerk","Biogasanlagenbau",{"stahl":30,"betonfertigteile":50,"pumpe":4,"elektromotor":4},4000,profil=MB,tech="biogas",umsatz=40),
   wg="maschinen", branche="maschinenbau", staat=st(0.01,1000000,1.0,{1990:0.2,2010:1.0,2026:0.8}), **INDUSTRIE),
 E("blockheizkraftwerk","Blockheizkraftwerk",300000,"stueck",8000,None,
   R("bhkwwerk","Blockheizkraftwerkswerk",{"dieselmotor":1,"kupferdraht":0.5,"stahl":3},1200,profil=MB,tech="kraft_waerme_kopplung",umsatz=40),
   wg="maschinen", branche="maschinenbau", staat=st(0.01,300000,1.0,{1975:0.3,2000:1.0,2026:1.0}), **INDUSTRIE),
 E("klaeranlage","Klärwerkstechnik",2000000,"stueck",300000,None,
   R("klaertechnikwerk","Klärtechnikwerk",{"pumpe":10,"stahl":50,"elektromotor":10,"betonfertigteile":200},8000,profil=MB,tech="belebtschlamm",umsatz=60,tage=30),
   wg="maschinen", branche="maschinenbau", staat=st(0.03,2000000,1.0,{1914:0.2,1960:1.0,2026:1.2}), **INDUSTRIE),
 E("rauchgasreinigung","Rauchgasreinigung",10000000,"stueck",500000,None,
   R("rauchgaswerk","Rauchgasreinigungswerk",{"stahl":200,"pumpe":10,"kalkstein":500,"elektromotor":5},40000,profil=MB,tech="rauchgasentschwefelung",umsatz=100,tage=30),
   wg="maschinen", branche="maschinenbau", staat=st(0.02,10000000,1.0,{1974:0.3,1990:1.0,2026:0.6}), **INDUSTRIE),
 E("abgaskatalysator","Abgaskatalysator",300,"stueck",5,("e","automobil",0.08,15000,1.0,1.0,GB),
   R("katalysatorwerk","Katalysatorwerk",{"stahl":0.003,"ton":0.002,"nickel":0.0005},1.2,profil=CH,tech="abgaskatalysator",umsatz=40),
   wg="fahrzeuge", branche="fahrzeuge"),
 E("muellverbrennungsanlage","Müllverbrennungsanlage",30000000,"stueck",2000000,None,
   R("muellanlagenbau","Müllanlagenbau",{"stahl":500,"industriekessel":4,"elektromotor":10,"betonfertigteile":1000},120000,profil=MB,tech="muellverbrennung",umsatz=200,tage=60),
   wg="maschinen", branche="maschinenbau", staat=st(0.01,30000000,1.0,{1960:0.3,1990:1.0,2026:1.0}), **INDUSTRIE),
 E("oelheizung","Ölheizung",3000,"stueck",200,("g",20,0.4,15000,1.2,1.0,GB),
   R("oelheizungswerk","Ölheizungswerk",{"blech":0.1,"kleinmotor":1,"stahl":0.05},12,profil=MB,tech="oelfeuerung",umsatz=30)),
 E("gasheizung","Gasheizkessel",2500,"stueck",60,("g",20,0.5,15000,1.2,1.0,GB),
   R("gasheizungswerk","Gasheizungswerk",{"blech":0.06,"kupfer":0.005,"kleinmotor":1},10,profil=MB,tech="gasheizkessel",umsatz=40)),
 E("kohleofen","Kohleofen",300,"stueck",100,("g",20,0.7,2500,1.1,1.0,G),
   R("ofenwerk","Ofenfabrik",{"stahl":0.08,"mauerziegel":10},3,profil=ME,umsatz=10,rname="Öfen gießen und setzen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("pelletheizung","Pelletheizung",15000,"stueck",400,("g",20,0.05,40000,1.3,1.2,GB),
   R("pelletheizungswerk","Pelletheizungswerk",{"blech":0.15,"kleinmotor":2,"mikrochip":20,"stahl":0.05},60,profil=MB,tech="pelletfeuerung",umsatz=30)),
 # Medizintechnik
 E("roentgengeraet","Röntgengerät",200000,"stueck",1000,None,
   R("roentgenwerk","Röntgengerätewerk",{"glas":0.05,"kupferdraht":0.2,"stahl":0.5},800,tech="roentgentechnik",umsatz=40),
   staat=st(0.02,200000,1.0,{1905:0.3,1950:1.0,2026:1.2}), **INDUSTRIE),
 E("ekg_geraet","EKG-Gerät",5000,"stueck",10,None,
   R("ekgwerk","EKG-Gerätewerk",{"radioroehre":20,"kupferdraht":0.002,"stahl":0.01,"glas":0.001},20,tech="elektrokardiograf",umsatz=20),
   staat=st(0.005,5000,1.0,{1910:0.2,1960:1.0,2026:1.2}), **INDUSTRIE),
 E("blutdruckmessgeraet","Blutdruckmessgerät",50,"stueck",0.5,("g",8,0.4,8000,1.1,1.0,GB),
   R("blutdruckwerk","Blutdruckmessgerätewerk",{"gummi":0.0002,"glas":0.0002,"messing":0.0002},0.3,tech="blutdruckmessung",umsatz=10)),
 E("fieberthermometer","Fieberthermometer",5,"stueck",0.02,("g",5,0.8,2000,1.0,1.0,G),
   R("thermometerwerk","Thermometerglashütte",{"glas":0.00002},0.03,profil=CH,umsatz=5,rname="Thermometer blasen und eichen")),
 E("hoergeraet","Hörgerät",1500,"stueck",0.05,("g",5,0.03,15000,1.2,1.2,GB),
   R("hoergeraetewerk","Hörgerätewerk",{"transistor":5,"kunststoffwaren":0.01,"trockenbatterie":1,"lautsprecher":1},6,tech="transistorhoergeraet",umsatz=20)),
 E("brille","Brille",150,"stueck",0.03,("g",4,0.4,3000,1.0,0.8,G),
   R("brillenwerk","Optische Werkstätte",{"glas":0.00005,"messing":0.00003},0.8,profil=ME,umsatz=10,rname="Brillengläser schleifen und fassen")),
 E("rollstuhl","Rollstuhl",1000,"stueck",15,None,
   R("rollstuhlwerk","Rollstuhlwerk",{"stahl":0.015,"gummi":0.0005,"stoff":0.001},5,profil=MB,tech="faltrollstuhl",umsatz=10),
   staat=st(0.003,1000,2.0,{1933:0.5,1970:1.0,2026:1.5}), **INDUSTRIE),
 E("krankenhausbett","Krankenhausbett",2000,"stueck",80,None,
   R("krankenbettwerk","Krankenbettwerk",{"stahl":0.05,"matratze":1},8,profil=MB,tech="verstellbares_krankenbett",umsatz=20),
   wg="moebel_hausrat", staat=st(0.02,2000,1.5,{1909:0.3,1950:1.0,2026:1.0}), **INDUSTRIE),
 E("beatmungsgeraet","Beatmungsgerät",30000,"stueck",40,None,
   R("beatmungswerk","Beatmungsgerätewerk",{"kompressor":1,"stahl":0.05,"kupferdraht":0.002},120,tech="beatmung",umsatz=30),
   staat=st(0.005,30000,2.0,{1952:0.3,1980:1.0,2026:1.5}), **INDUSTRIE),
 E("dialysegeraet","Dialysegerät",30000,"stueck",80,None,
   R("dialysewerk","Dialysegerätewerk",{"pumpe":2,"kunststoffwaren":20,"mikrochip":100,"stahl":0.03},120,tech="haemodialyse",umsatz=30),
   staat=st(0.005,30000,1.0,{1964:0.2,1985:1.0,2026:1.5}), **INDUSTRIE),
 E("herzschrittmacher","Herzschrittmacher",5000,"stueck",0.03,None,
   R("schrittmacherwerk","Schrittmacherwerk",{"transistor":20,"titan":0.00005,"trockenbatterie":1},20,tech="herzschrittmacher",umsatz=20),
   staat=st(0.003,5000,1.0,{1960:0.2,1980:1.0,2026:1.5}), **INDUSTRIE),
 E("ultraschallgeraet","Ultraschallgerät",80000,"stueck",100,None,
   R("ultraschallwerk","Ultraschallgerätewerk",{"mikrochip":2000,"bildroehre":1,"kunststoffwaren":20,"leiterplatte":10},300,tech="sonografie",umsatz=40),
   staat=st(0.005,80000,1.0,{1970:0.2,1990:1.0,2026:1.5}), **INDUSTRIE),
 E("computertomograph","Computertomograph",1500000,"stueck",2000,None,
   R("ctwerk","Computertomographenwerk",{"mikrochip":50000,"stahl":3,"kupferdraht":0.5},6000,tech="computertomografie",umsatz=100),
   staat=st(0.005,1500000,1.0,{1972:0.2,1990:1.0,2026:1.5}), **INDUSTRIE),
 E("mrt","Magnetresonanztomograph",3000000,"stueck",10000,None,
   R("mrtwerk","Magnetresonanzwerk",{"kupferdraht":10,"stahl":20,"mikrochip":50000,"industriegase":2},12000,tech="magnetresonanz",umsatz=150),
   staat=st(0.005,3000000,1.0,{1980:0.2,2000:1.0,2026:1.5}), **INDUSTRIE),
 E("einwegspritze","Einwegspritzen (100 Stück)",20,"stueck",0.6,None,
   R("spritzenwerk","Spritzenwerk",{"polypropylen":0.0005,"stahl":0.00005},0.08,profil=CH,tech="einwegspritze",umsatz=30),
   wg="chemie", branche="pharma", staat=st(0.05,20,1.0,{1956:0.3,1980:1.0,2026:1.5}), **INDUSTRIE),
 E("hueftprothese","Hüftprothese",5000,"stueck",0.4,None,
   R("prothesenwerk","Prothesenwerk",{"titan":0.0003,"polyethylen":0.0001,"stahl":0.0003},20,profil=ME,tech="hueftendoprothese",umsatz=20),
   staat=st(0.005,5000,1.0,{1962:0.2,1990:1.0,2026:1.5}), **INDUSTRIE),
 E("zahnimplantat","Zahnimplantat",1500,"stueck",0.01,("g",20,0.05,40000,1.3,1.3,GB),
   R("implantatwerk","Implantatwerk",{"titan":0.00001},6,profil=ME,tech="zahnimplantat",umsatz=20)),
 E("rettungswagen","Rettungswagen",150000,"stueck",3500,None,
   R("rettungswagenwerk","Rettungswagenwerk",{"lieferwagen":1,"stahl":0.2,"kupferdraht":0.01},500,profil=MB,tech="rettungswagen",umsatz=30),
   wg="fahrzeuge", branche="fahrzeuge", staat=st(0.01,150000,1.5,{1925:0.3,1970:1.0,2026:1.2}), **INDUSTRIE),
 # Freizeit
 E("e_bike","Elektrofahrrad",2500,"stueck",25,("g",8,0.3,25000,1.3,1.2,GB),
   R("ebikewerk","Elektrofahrradwerk",{"fahrrad":1,"traktionsbatterie":0.5,"kleinmotor":1,"mikrochip":10},8,profil=MB,tech="pedelec",umsatz=40),
   wg="fahrzeuge", branche="fahrzeuge"),
 E("modelleisenbahn","Modelleisenbahn",200,"stueck",2,("g",15,0.1,10000,1.4,1.4,LX),
   R("modellbahnwerk","Modellbahnwerk",{"blech":0.002,"kleinmotor":1,"messing":0.0003},1.2,profil=ME,umsatz=10,rname="Modellbahnen bauen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("synthesizer","Synthesizer",1500,"stueck",12,("g",12,0.02,30000,1.4,1.4,LX),
   R("synthesizerwerk","Synthesizerwerk",{"transistor":200,"schnittholz":0.005,"kunststoffwaren":2},6,tech="synthesizer",umsatz=10),
   branche="konsumgueter"),
 E("klavier","Klavier",6000,"stueck",250,("g",40,0.08,15000,1.5,1.5,LX),
   R("klavierbau","Klavierbau",{"schnittholz":0.2,"stahl":0.03,"leim":0.005},40,profil=HB,umsatz=10,rname="Klaviere bauen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("gitarre","Gitarre",300,"stueck",2,("g",20,0.1,5000,1.3,1.3,GB),
   R("gitarrenbau","Gitarrenbau",{"schnittholz":0.003,"stahl":0.0001},2,profil=HB,umsatz=5,rname="Gitarren bauen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("e_gitarre","E-Gitarre",800,"stueck",4,("g",20,0.03,15000,1.4,1.4,LX),
   R("egitarrenwerk","E-Gitarrenwerk",{"schnittholz":0.004,"kupferdraht":0.0001,"stahl":0.0003},4,profil=HB,tech="elektrogitarre",umsatz=10),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("teddybaer","Plüschtier",40,"stueck",0.4,("g",5,0.3,4000,1.2,1.2,GB),
   R("plueschwerk","Plüschtierwerk",{"wollstoff":0.0003,"stoff":0.0002},0.3,profil=TX,tech="pluesch_spielzeug",umsatz=10),
   wg="textilien", branche="textil"),
 E("ski","Ski",300,"stueck",4,("g",8,0.05,8000,1.4,1.4,LX),
   R("skiwerk","Skiwerk",{"schnittholz":0.004,"stahl":0.0005},2,profil=HB,umsatz=5,rname="Ski bauen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("snowboard","Snowboard",400,"stueck",3,("g",6,0.02,25000,1.4,1.3,LX),
   R("snowboardwerk","Snowboardwerk",{"polyesterfaser":0.002,"schnittholz":0.002,"stahl":0.0005},2,profil=HB,tech="snowboard",umsatz=10),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("tennisschlaeger","Tennisschläger",150,"stueck",0.4,("g",5,0.03,8000,1.3,1.3,LX),
   R("schlaegerwerk","Schlägerwerk",{"schnittholz":0.0004,"garn":0.0001},1,profil=HB,umsatz=5,rname="Schläger bauen und bespannen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("fussball","Fußball",30,"stueck",0.4,("g",2,0.2,3000,1.2,1.2,GB),
   R("ballwerk","Ballnäherei",{"leder":0.3,"gummi":0.0001},0.3,profil=TX,umsatz=5,rname="Bälle nähen"),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("zelt","Zelt",200,"stueck",5,("g",10,0.1,6000,1.2,1.2,GB),
   R("zeltwerk","Zeltnäherei",{"stoff":0.004,"stahl":0.001},1.2,profil=TX,umsatz=5,rname="Zelte nähen"),
   wg="textilien", branche="textil"),
 E("laufband","Laufband",2000,"stueck",80,("g",10,0.03,35000,1.4,1.3,LX),
   R("laufbandwerk","Laufbandwerk",{"elektromotor":1,"stahl":0.04,"mikrochip":10,"gummi":0.005},8,profil=MB,tech="laufband",umsatz=20),
   wg="maschinen", branche="maschinenbau"),
 E("surfbrett","Surfbrett",600,"stueck",6,("g",8,0.01,25000,1.4,1.3,LX),
   R("surfbrettwerk","Surfbrettwerk",{"polyesterfaser":0.004,"polystyrol":0.003},3,profil=CH,tech="kunststoff_surfbrett",umsatz=5),
   wg="moebel_hausrat", branche="konsumgueter"),
 E("vr_brille","VR-Brille",400,"stueck",0.5,("g",4,0.05,35000,1.4,1.3,GB),
   R("vrwerk","VR-Brillenwerk",{"lcd_panel":0.005,"mikrochip":30,"kunststoffwaren":0.3,"lithium_ionen_akku":2},1.5,tech="virtuelle_realitaet",umsatz=40)),
]

EINFACH = {"uranerz", "wasserturbine", "kohleofen", "fieberthermometer", "brille", "blutdruckmessgeraet", "klavier", "gitarre", "e_gitarre", "teddybaer", "ski", "tennisschlaeger", "fussball", "zelt", "modelleisenbahn", "roentgengeraet", "solarkollektor", "zahnimplantat", "hueftprothese"}
for _p in P5:
    # Plants, medical devices and electronics reach five or six levels (§17.2).
    if _p["id"] not in EINFACH:
        _p["komplex"] = True
