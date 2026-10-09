# Spezifikation P2a–P2b (Felder wie in p1/spec.py). Preise um 1900 bzw. zum Erfindungsjahr,
# USD Kaufkraft 2026 (nominal × rund 30). X = Jahreseinkommen je Kopf, bei dem die Hälfte
# einer Schicht kauft. Staatsnachfrage: (Einheiten je Mio. USD BIP, Kriegsfaktor[, Verlauf]).
#
# Maschinen und Bauteile kaufen in WSim Staat und Wirtschaft über die Staatsnachfrage
# (Investitionen der Betriebe sind keine Nachfrage nach Gütern). Bauteile (Kugellager,
# Getriebe) gehen in die Maschinen ein.

LM, TX, HB, CH, ME, MB, FARM = "lm", "tx", "hb", "ch", "me", "mb", "farm"
G, GB, LX = "grundbedarf", "gebrauchsgut", "luxus"


def P(**k): return k


def E(id, name, preis, einheit, gewicht, nach, rez, wg="chemie", branche="chemie", tk="stueckgut", art="endprodukt", verw="konsum", **k):
    d = dict(id=id, name=name, art=art, branche=branche, einheit=einheit, wg=wg, tk=tk, preis=preis, verw=verw, nach=nach, rez=rez)
    if gewicht is not None: d["gewicht"] = gewicht
    d.update(k)
    return d


def R(anlage, aname, ein, h, profil=CH, tech=None, menge=1, umsatz=5, typ="werk", rname=None, neben=None, tage=1):
    r = dict(anlage=(anlage, aname, typ, umsatz), ein=ein, h=h, profil=profil, menge=menge, tage=tage)
    if tech: r["tech"] = tech
    if rname: r["rname"] = rname
    if neben: r["neben"] = neben
    return r


def H(id, name, preis, einheit, rez, wg="chemie", branche="chemie", tk="schuettgut", gewicht=None, **k):
    """A semi-finished good for industry."""
    return E(id, name, preis, einheit, gewicht, None, rez, wg=wg, branche=branche, tk=tk, art="halbzeug", verw="industrie", **k)


# ---------------------------------------------------------------- P2a Chemie, Pharma, Kosmetik, Kunststoffwaren
P2A = [
 # Rohstoffe
 P(id="phosphat", name="Rohphosphat", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=90, verw="industrie",
   rez=dict(anlage=("phosphatgrube","Phosphatgrube","foerderstaette",3), h=6, profil=FARM, abbau=True, rname="Phosphat abbauen"),
   index={1900:1.0,1930:3.0,1950:5.0,1970:15.0,1990:22.0,2010:26.0,2026:30.0}, pacht=0.3,
   dep=[("phosphat_usa","Phosphatfelder Florida","USA",2.5e6,0.8),("phosphat_tun","Phosphate von Gafsa","TUN",1.0e6,0.9),("phosphat_dza","Phosphate Algerien","DZA",5e5,1.0),
        ("phosphat_mar","Phosphate von Khouribga","MAR",1.2e6,0.8),("phosphat_rus","Apatit der Kola-Halbinsel","RUS",6e5,1.1),("phosphat_chn","Phosphat Yunnan","CHN",6e5,1.0),
        ("phosphat_jor","Phosphat Jordanien","XLE",3e5,0.9),("phosphat_pac","Phosphatinseln im Pazifik","XOZ",4e5,1.0)]),
 P(id="kalisalz", name="Kalisalz", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=80, verw="industrie",
   rez=dict(anlage=("kalibergwerk","Kalibergwerk","foerderstaette",3), h=10, profil=FARM, abbau=True, rname="Kalisalz fördern"),
   index={1900:1.0,1930:2.0,1950:3.0,1970:8.0,1990:9.0,2010:12.0,2026:13.0}, pacht=0.3,
   dep=[("kali_deu","Kalireviere Staßfurt und Werra","DEU",3.0e6,0.9),("kali_fra","Kalibecken im Elsass","FRA",8e5,1.0),("kali_rus","Kali Solikamsk","RUS",1.0e6,1.0),
        ("kali_blr","Kali Soligorsk","BLR",6e5,1.0),("kali_can","Kali Saskatchewan","CAN",1.2e6,0.8),("kali_usa","Kali Carlsbad","USA",5e5,1.0)]),
 # Halbzeuge
 H("schwefelsaeure","Schwefelsäure",450,"t",R("schwefelsaeurefabrik","Schwefelsäurefabrik",{"schwefel":0.34},0.6,rname="Schwefel rösten und Säure kontaktieren")),
 H("chlor","Chlor",700,"t",R("chloralkalielektrolyse","Chloralkali-Elektrolyse",{"salz":1.7},1.2,rname="Sole elektrolysieren",neben={"natronlauge":1.1}), tk="gas"),
 P(id="natronlauge", name="Natronlauge", art="halbzeug", branche="chemie", einheit="t", wg="chemie", tk="fluessig", preis=400, verw="industrie", sm=400),
 H("benzol","Benzol und Teerprodukte",600,"t",R("kokerei_chemie","Teerdestillation",{"kohle":3.0},2.0,rname="Kohle verkoken und Teer destillieren"), tk="fluessig"),
 H("farbstoffe","Farbstoffe",8000,"t",R("farbenfabrik","Teerfarbenfabrik",{"benzol":0.8,"schwefelsaeure":0.5},40,rname="Farbstoffe synthetisieren"), tk="stueckgut"),
 H("pharmagrundstoffe","Pharmazeutische Wirkstoffe",200,"kg",R("wirkstoffwerk","Wirkstoffwerk",{"benzol":0.003,"schwefelsaeure":0.002},1.0,rname="Wirkstoffe synthetisieren"), branche="pharma", tk="stueckgut"),
 H("tenside","Tenside",2000,"t",R("tensidwerk","Tensidwerk",{"ethylen":0.5,"schwefelsaeure":0.5},6,tech="synthetische_tenside",rname="Fettalkohole sulfonieren"), tk="fluessig"),
 H("pvc","PVC",2500,"t",R("pvc_werk","PVC-Werk",{"ethylen":0.45,"chlor":0.6},6,tech="pvc_herstellung",rname="Vinylchlorid polymerisieren")),
 H("polystyrol","Polystyrol",2500,"t",R("polystyrolwerk","Polystyrolwerk",{"ethylen":0.3,"benzol":0.8},6,tech="polystyrol",rname="Styrol polymerisieren")),
 H("polypropylen","Polypropylen",2000,"t",R("polypropylenwerk","Polypropylenwerk",{"ethylen":1.05},5,tech="polypropylen",rname="Propylen polymerisieren")),
 # Dünger und Pflanzenschutz (Staat steht für die Landwirtschaft, wie beim Stickstoffdünger)
 E("phosphatduenger","Phosphatdünger",400,"t",None,None,R("superphosphatwerk","Superphosphatwerk",{"phosphat":1.0,"schwefelsaeure":0.4},1.0,rname="Phosphat aufschließen"),
   tk="schuettgut", verw="industrie", staat=(0.6,1.0,{1900:1.0,1950:2.5,1970:6.0,1990:6.0,2010:5.0,2026:5.0})),
 E("kaliduenger","Kalidünger",200,"t",None,None,R("kalifabrik","Kalifabrik",{"kalisalz":1.6},0.8,rname="Kalisalz aufbereiten"),
   tk="schuettgut", verw="industrie", staat=(0.6,1.0,{1900:1.0,1950:2.5,1970:6.0,1990:6.0,2010:5.0,2026:5.0})),
 E("pflanzenschutzmittel","Pflanzenschutzmittel",12,"kg",None,None,R("pflanzenschutzwerk","Pflanzenschutzwerk",{"chlor":0.0008,"benzol":0.0008},0.4,tech="synthetische_insektizide",rname="Insektizide formulieren"),
   verw="industrie", staat=(40,1.0,{1939:1.0,1960:3.0,1980:5.0,2026:5.0})),
 # Wasch- und Reinigungsmittel, Körperpflege
 E("seife","Seife",5.0,"kg",None,("v",2,600,0.5,0.6,G),R("seifensiederei","Seifensiederei",{"speiseoel":0.7,"natronlauge":0.00015},0.08,rname="Seife sieden"), wg="chemie", branche="konsumgueter"),
 E("waschmittel","Waschmittel",5.0,"kg",None,("v",4,1500,0.5,0.7,G),R("waschmittelwerk","Waschmittelwerk",{"soda":0.0004,"seife":0.3},0.05,tech="waschpulver",rname="Waschpulver mischen"), branche="konsumgueter"),
 E("spuelmittel","Spülmittel",3.0,"l",None,("v",3,6000,0.6,0.8,G),R("spuelmittelwerk","Spülmittelwerk",{"tenside":0.0002},0.02,tech="fluessiges_spuelmittel",rname="Spülmittel abfüllen"), branche="konsumgueter"),
 E("haushaltsreiniger","Haushaltsreiniger",4.0,"l",None,("v",3,6000,0.6,0.8,G),R("reinigerwerk","Reinigerwerk",{"tenside":0.0001,"chlor":0.00005},0.02,tech="haushaltsreiniger",rname="Reiniger mischen und abfüllen"), branche="konsumgueter"),
 E("zahnpasta","Zahnpasta",3.0,"stueck",0.12,("v",4,3000,0.5,0.7,G),R("zahnpastawerk","Zahnpastawerk",{"kalkstein":0.00005,"zinn":0.000005},0.03,rname="Zahnpasta anrühren und in Tuben füllen"), branche="konsumgueter"),
 E("shampoo","Shampoo",8.0,"l",None,("v",2,8000,0.6,0.9,G),R("shampoowerk","Shampoowerk",{"seife":0.2,"farbstoffe":0.000002},0.05,tech="fluessigshampoo",rname="Shampoo mischen"), branche="konsumgueter"),
 E("hautcreme","Hautcreme",6.0,"stueck",0.15,("v",3,8000,0.6,0.9,G),R("cremewerk","Cremewerk",{"speiseoel":0.04,"glas":0.00008},0.04,tech="emulsionscreme",rname="Creme emulgieren"), branche="konsumgueter"),
 E("parfuem","Parfüm",40.0,"stueck",0.15,("v",0.5,20000,0.8,1.2,LX),R("parfuemerie","Parfümerie",{"spirituosen":0.04,"glas":0.0001},0.5,rname="Duftöle mischen und abfüllen"), branche="konsumgueter"),
 E("dekorative_kosmetik","Dekorative Kosmetik",10.0,"stueck",0.05,("v",3,12000,0.8,1.1,LX),R("kosmetikwerk","Kosmetikwerk",{"farbstoffe":0.00001,"speiseoel":0.01},0.1,tech="lippenstifthuelse",rname="Lippenstifte und Puder herstellen"), branche="konsumgueter"),
 E("deodorant","Deodorant",4.0,"stueck",0.1,("v",4,10000,0.7,0.9,G),R("deowerk","Deo-Abfüllwerk",{"spirituosen":0.02,"polyethylen":0.00002},0.03,tech="deoroller",rname="Deodorant abfüllen"), branche="konsumgueter"),
 # Arzneimittel
 E("schmerzmittel","Schmerzmittel",5.0,"stueck",0.03,("v",4,2000,0.4,0.6,G),R("tablettenwerk","Tablettenwerk",{"pharmagrundstoffe":0.01},0.03,rname="Tabletten pressen"), branche="pharma", staat=(60,1.5)),
 E("impfstoff","Impfstoff",15.0,"stueck",0.01,("v",0.3,3000,0.4,0.6,G),R("impfstoffwerk","Impfstoffwerk",{"pharmagrundstoffe":0.002,"glas":0.000005},0.15,tech="impfstoffproduktion",rname="Impfstoff züchten und abfüllen"), branche="pharma", staat=(150,2.0)),
 E("insulin","Insulin",30.0,"stueck",0.02,("v",0.3,6000,0.3,0.5,G),R("insulinwerk","Insulinwerk",{"pharmagrundstoffe":0.002,"schlachtvieh":0.00002},0.3,tech="insulin",rname="Insulin gewinnen"), branche="pharma"),
 E("vitaminpraeparate","Vitaminpräparate",8.0,"stueck",0.05,("v",1,15000,0.7,1.0,G),R("vitaminwerk","Vitaminwerk",{"pharmagrundstoffe":0.005},0.05,tech="vitaminsynthese",rname="Vitamine synthetisieren und pressen"), branche="pharma"),
 E("antibabypille","Antibabypille",15.0,"stueck",0.01,("v",2,10000,0.5,0.7,G),R("hormonwerk","Hormonwerk",{"pharmagrundstoffe":0.001},0.1,tech="hormonpraeparate",rname="Hormonpräparate herstellen"), branche="pharma"),
 E("herzkreislaufmittel","Herz-Kreislauf-Mittel",20.0,"stueck",0.03,("v",2,15000,0.4,0.7,G),R("herzmittelwerk","Werk für Herz-Kreislauf-Mittel",{"pharmagrundstoffe":0.004},0.15,tech="betablocker",rname="Herz-Kreislauf-Mittel herstellen"), branche="pharma", staat=(40,1.0)),
 E("biopharmazeutika","Biopharmazeutika",300.0,"stueck",0.02,("v",0.1,30000,0.4,0.8,G),R("biotechwerk","Biotechnologiewerk",{"pharmagrundstoffe":0.002,"zucker":0.001},2.0,tech="gentechnische_wirkstoffe",rname="Wirkstoffe in Zellkulturen gewinnen"), branche="pharma", staat=(2,1.0)),
 # Kunststoffwaren, Farben, Klebstoffe
 E("kunststoffrohre","Kunststoffrohre",4000,"t",None,None,R("rohrextrusion","Rohrextrusion",{"pvc":1.05},6,tech="kunststoffrohr",rname="PVC-Rohre extrudieren"),
   wg="moebel_hausrat", branche="bau", verw="industrie", staat=(0.05,1.0,{1936:1.0,1960:4.0,1980:8.0,2026:10.0})),
 E("bodenbelag","Kunststoff-Bodenbelag",15.0,"m2",2.0,("v",0.3,8000,0.7,0.9,G),R("bodenbelagwerk","Bodenbelagwerk",{"pvc":0.002},0.05,tech="kunststoffbodenbelag",rname="Bodenbeläge kalandrieren"), wg="moebel_hausrat", branche="bau"),
 E("schallplatte","Schallplatte",20.0,"stueck",0.15,("v",1,10000,0.8,1.1,LX),R("presswerk","Schallplattenpresswerk",{"pvc":0.00015},0.15,tech="vinylschallplatte",rname="Schallplatten pressen"), wg="moebel_hausrat", branche="konsumgueter"),
 E("kunststoffspielzeug","Kunststoffspielzeug",10.0,"stueck",0.3,("v",3,8000,0.7,1.0,G),R("spielzeugfabrik","Spielzeugfabrik",{"polystyrol":0.0003},0.1,profil=MB,tech="kunststoffspielzeug",rname="Spielzeug spritzgießen"), wg="moebel_hausrat", branche="konsumgueter"),
 E("verpackungsschaum","Verpackungsschaum",60.0,"m3",20.0,None,R("schaumstoffwerk","Schaumstoffwerk",{"polystyrol":0.02},0.5,tech="expandiertes_polystyrol",rname="Polystyrol aufschäumen"),
   wg="moebel_hausrat", branche="bau", verw="industrie", staat=(4,1.0,{1951:1.0,1970:4.0,2000:8.0,2026:8.0})),
 E("gartenschlauch","Gartenschlauch",25.0,"stueck",2.0,("g",8,0.5,8000,0.7,0.9,GB),R("schlauchwerk","Schlauchwerk",{"pvc":0.002},0.2,tech="pvc_schlauch",rname="Schläuche extrudieren"), wg="moebel_hausrat", branche="konsumgueter"),
 E("muellbeutel","Müllbeutel",3.0,"stueck",0.2,("v",10,6000,0.6,0.8,G),R("beutelwerk","Beutelwerk",{"polyethylen":0.0002},0.02,tech="muellbeutel",rname="Folie schweißen und aufrollen"), wg="moebel_hausrat", branche="konsumgueter"),
 E("tragetasche","Kunststoff-Tragetasche",0.1,"stueck",0.006,("v",100,4000,0.5,0.6,G),R("taschenwerk","Tragetaschenwerk",{"polyethylen":0.000006},0.001,tech="plastiktuete",rname="Tragetaschen schweißen"), wg="moebel_hausrat", branche="konsumgueter"),
 E("farbe_lack","Farben und Lacke",8.0,"l",None,("v",1,3000,0.6,0.8,G),R("lackfabrik","Lackfabrik",{"oelsaaten":0.0006,"farbstoffe":0.00002},0.05,rname="Lacke anreiben"), branche="chemie", staat=(30,1.5)),
 E("klebstoff","Klebstoff",15.0,"kg",None,("v",0.2,8000,0.6,0.8,G),R("klebstoffwerk","Klebstoffwerk",{"benzol":0.0005},0.1,tech="alleskleber",rname="Klebstoff lösen und abfüllen"), branche="chemie", staat=(5,1.0)),
 E("fotofilm","Fotofilm",6.0,"stueck",0.03,("v",1,8000,0.8,1.1,LX),R("filmfabrik","Filmfabrik",{"zellstoff":0.00002,"farbstoffe":0.000002},0.05,rname="Film begießen und konfektionieren"), branche="chemie"),
]

# ---------------------------------------------------------------- P2b Metallwaren, Werkzeuge, Maschinen
P2B = [
 P(id="zinkerz", name="Zinkerz", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=120, verw="industrie",
   rez=dict(anlage=("zinkgrube","Zinkgrube","foerderstaette",3), h=12, profil=FARM, abbau=True, rname="Zinkerz fördern"),
   index={1900:1.0,1930:2.0,1950:2.5,1970:5.0,1990:6.5,2010:10.0,2026:11.0}, pacht=0.3,
   dep=[("zink_aus","Broken Hill","AUS",4e5,0.8,6e7),("zink_usa","Tri-State-Revier","USA",5e5,0.9,6e7),("zink_pol","Oberschlesisches Zinkrevier","POL",4e5,1.0,5e7),
        ("zink_mex","Zinkgruben Mexiko","MEX",2e5,0.9,4e7),("zink_can","Sullivan-Grube","CAN",2e5,0.9,4e7),("zink_per","Zink der Anden","PER",1.5e5,0.9,5e7),
        ("zink_chn","Zinkgruben Yunnan","CHN",2e5,1.0,8e7),("zink_rus","Zinkerz Ural","RUS",1.5e5,1.1,4e7),("zink_kaz","Zinkerz Kasachstan","KAZ",1e5,1.0,4e7)]),
 H("zink","Zink",3000,"t",R("zinkhuette","Zinkhütte",{"zinkerz":2.0,"kohle":1.0},8,profil=ME,rname="Zink rösten und reduzieren"), wg="nichteisenmetalle", branche="metallurgie", tk="stueckgut"),
 H("messing","Messing",13000,"t",R("messingwerk","Messingwerk",{"kupfer":0.65,"zink":0.36},12,profil=ME,rname="Messing gießen und walzen"), wg="nichteisenmetalle", branche="metallurgie", tk="stueckgut"),
 H("kugellager","Kugellager",15.0,"stueck",R("waelzlagerwerk","Wälzlagerwerk",{"stahl":0.0012},0.2,profil=ME,rname="Kugeln schleifen und Lager montieren"), wg="maschinen", branche="maschinenbau", tk="stueckgut", gewicht=1.0),
 H("getriebe","Getriebe",600.0,"stueck",R("getriebewerk","Getriebewerk",{"stahl":0.05,"kugellager":2},6,profil=MB,rname="Zahnräder fräsen und Getriebe montieren"), wg="maschinen", branche="maschinenbau", tk="stueckgut", gewicht=60),
 H("dieselmotor","Dieselmotor",3000.0,"stueck",R("dieselmotorenwerk","Dieselmotorenwerk",{"stahl":0.4,"kugellager":4},40,profil=MB,rname="Dieselmotoren bauen"), wg="maschinen", branche="maschinenbau", tk="stueckgut", gewicht=500),
 # Baustahl und Rohre (Staat steht für Bau und Leitungen)
 E("stahlrohre","Stahlrohre",900,"t",None,None,R("rohrwerk","Rohrwalzwerk",{"stahl":1.05},4,profil=ME,rname="Rohre nahtlos walzen"),
   wg="eisen_stahl", branche="metallurgie", verw="industrie", staat=(0.4,1.5)),
 E("stahltraeger","Stahlträger",800,"t",None,None,R("traegerwalzwerk","Trägerwalzwerk",{"stahl":1.03},3,profil=ME,rname="Träger walzen"),
   wg="eisen_stahl", branche="bau", verw="industrie", staat=(0.8,1.5)),
 # Metallwaren für Haushalte
 E("schneidwaren","Messer und Scheren",8.0,"stueck",0.2,("g",10,2,1000,0.6,0.7,GB),R("schneidwarenfabrik","Schneidwarenfabrik",{"stahl":0.0002},0.1,profil=ME,rname="Klingen schmieden und schleifen"), wg="metallwaren", branche="konsumgueter"),
 E("rasierklingen","Rasierklingen",0.3,"stueck",0.002,("v",50,3000,0.5,0.6,G),R("klingenfabrik","Klingenfabrik",{"stahl":0.000003},0.003,profil=ME,tech="wegwerfrasierklinge",rname="Klingen stanzen und härten"), wg="metallwaren", branche="konsumgueter"),
 E("schloesser","Schlösser und Beschläge",10.0,"stueck",0.5,("v",0.5,2000,0.5,0.7,G),R("schlossfabrik","Schlossfabrik",{"messing":0.0003,"stahl":0.0002},0.12,profil=ME,rname="Schlösser montieren"), wg="metallwaren", branche="konsumgueter", staat=(50,1.0)),
 E("armaturen","Sanitärarmaturen",40.0,"stueck",1.5,("g",15,2,6000,0.6,0.8,GB),R("armaturenwerk","Armaturenwerk",{"messing":0.0012},0.4,profil=ME,rname="Armaturen gießen und verchromen"), wg="metallwaren", branche="bau", staat=(10,1.0)),
 E("heizkoerper","Heizkörper",120.0,"stueck",30,("g",30,4,8000,0.6,0.9,GB),R("heizkoerperwerk","Heizkörperwerk",{"stahl":0.03},1.0,profil=ME,rname="Heizkörper gießen oder schweißen"), wg="metallwaren", branche="bau", staat=(5,1.0)),
 E("badewanne","Badewanne",300.0,"stueck",100,("g",30,1,8000,0.7,0.9,GB),R("wannenwerk","Wannenwerk",{"stahl":0.1},2.5,profil=ME,rname="Wannen gießen und emaillieren"), wg="metallwaren", branche="bau"),
 E("taschenuhr","Taschenuhr",60.0,"stueck",0.1,("g",15,1,3000,0.8,1.0,LX),R("uhrenfabrik","Uhrenfabrik",{"messing":0.00005,"stahl":0.00002},1.0,profil=ME,rname="Uhrwerke montieren"), wg="metallwaren", branche="konsumgueter"),
 E("armbanduhr","Armbanduhr",60.0,"stueck",0.05,("g",10,1,4000,0.8,1.0,LX),R("armbanduhrenfabrik","Armbanduhrenfabrik",{"messing":0.00003,"stahl":0.00002,"leder":0.00001},1.0,profil=ME,tech="armbanduhr",rname="Armbanduhren montieren"), wg="metallwaren", branche="konsumgueter", ersetzt=["taschenuhr"]),
 E("quarzuhr","Quarzuhr",40.0,"stueck",0.05,("g",8,1,6000,0.8,1.0,G),R("quarzuhrenwerk","Quarzuhrenwerk",{"stahl":0.00002,"mikrochip":1},0.3,profil=MB,tech="quarzuhr",rname="Quarzuhren montieren"), wg="elektro", branche="konsumgueter"),
 E("wecker","Wecker",15.0,"stueck",0.4,("g",10,1,1500,0.6,0.8,GB),R("weckerfabrik","Weckerfabrik",{"messing":0.0002,"stahl":0.0002},0.25,profil=ME,rname="Wecker montieren"), wg="metallwaren", branche="konsumgueter"),
 # Werkzeuge und Geräte
 E("schreibmaschine","Schreibmaschine",600.0,"stueck",12,("g",15,0.2,20000,0.7,1.0,GB),R("schreibmaschinenfabrik","Schreibmaschinenfabrik",{"stahl":0.008,"gummi":0.0002},8,profil=MB,rname="Schreibmaschinen montieren"), wg="maschinen", branche="maschinenbau", staat=(0.08,1.0)),
 E("registrierkasse","Registrierkasse",1500.0,"stueck",40,None,R("kassenfabrik","Registrierkassenfabrik",{"stahl":0.03,"messing":0.002},20,profil=MB,rname="Kassen montieren"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.03,0.5)),
 E("elektrowerkzeug","Elektrowerkzeug",100.0,"stueck",2.0,("g",8,0.6,10000,0.7,1.0,GB),R("elektrowerkzeugfabrik","Elektrowerkzeugfabrik",{"kleinmotor":1,"stahl":0.001},1.0,profil=MB,tech="handbohrmaschine",rname="Bohrmaschinen montieren"), wg="maschinen", branche="maschinenbau", staat=(0.5,1.5)),
 E("rasenmaeher","Motorrasenmäher",300.0,"stueck",25,("g",10,0.4,15000,0.7,1.0,GB),R("rasenmaeherfabrik","Rasenmäherfabrik",{"kleinmotor":1,"stahl":0.01},3,profil=MB,tech="motorrasenmaeher",rname="Rasenmäher montieren"), wg="maschinen", branche="maschinenbau"),
 E("kettensaege","Motorkettensäge",400.0,"stueck",8,("g",10,0.05,20000,0.7,1.0,GB),R("kettensaegenfabrik","Kettensägenfabrik",{"kleinmotor":1,"stahl":0.005},3,profil=MB,tech="motorkettensaege",rname="Kettensägen montieren"), wg="maschinen", branche="maschinenbau", staat=(0.05,1.0)),
 # Maschinen und Anlagen
 E("werkzeugmaschine","Werkzeugmaschine",25000.0,"stueck",3000,None,R("werkzeugmaschinenfabrik","Werkzeugmaschinenfabrik",{"stahl":3,"getriebe":2,"elektromotor":2},300,profil=MB,rname="Drehbänke und Fräsmaschinen bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.01,2.0)),
 E("cnc_maschine","CNC-Bearbeitungszentrum",150000.0,"stueck",6000,None,R("cnc_werk","CNC-Maschinenwerk",{"stahl":4,"getriebe":2,"elektromotor":3,"mikrochip":20},1500,profil=MB,tech="cnc_steuerung",rname="Bearbeitungszentren bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.002,1.5,{1975:1.0,1990:3.0,2026:5.0})),
 E("industrieroboter","Industrieroboter",80000.0,"stueck",1000,None,R("roboterwerk","Roboterwerk",{"stahl":0.5,"kleinmotor":6,"transistor":50},800,profil=MB,tech="industrieroboter",rname="Roboter bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.0005,1.0,{1961:1.0,1980:6.0,2000:15.0,2026:30.0})),
 E("industriekessel","Industriekessel",30000.0,"stueck",10000,None,R("kesselschmiede","Kesselschmiede",{"stahl":10},300,profil=MB,rname="Kessel nieten und schweißen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.005,1.0)),
 E("pumpe","Kreiselpumpe",800.0,"stueck",80,None,R("pumpenwerk","Pumpenwerk",{"stahl":0.05,"kleinmotor":1},8,profil=MB,rname="Pumpen bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.15,1.0)),
 E("kompressor","Kompressor",6000.0,"stueck",300,None,R("kompressorenwerk","Kompressorenwerk",{"stahl":0.2,"elektromotor":1},30,profil=MB,rname="Kompressoren bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.03,1.0)),
 E("gabelstapler","Gabelstapler",15000.0,"stueck",3000,None,R("staplerwerk","Staplerwerk",{"motor":1,"stahl":1.5,"autoreifen":4},150,profil=MB,tech="gabelstapler",rname="Stapler montieren"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.005,1.0,{1923:1.0,1960:4.0,2026:6.0})),
 E("bagger","Bagger",60000.0,"stueck",20000,None,R("baumaschinenwerk","Baumaschinenwerk",{"stahl":15,"dieselmotor":1,"getriebe":2},600,profil=MB,rname="Bagger bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.004,1.5)),
 E("kran","Kran",50000.0,"stueck",25000,None,R("kranbau","Kranbau",{"stahl":20,"elektromotor":3},500,profil=MB,rname="Krane bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.003,1.5)),
 E("aufzug","Aufzug",20000.0,"stueck",3000,None,R("aufzugswerk","Aufzugswerk",{"stahl":3,"elektromotor":1,"draht":0.5},200,profil=MB,rname="Aufzüge bauen"), wg="maschinen", branche="bau", verw="industrie", staat=(0.006,0.5,{1900:1.0,1950:3.0,2000:6.0,2026:8.0})),
 E("landmaschinen","Landmaschinen",1500.0,"stueck",500,None,R("landmaschinenfabrik","Landmaschinenfabrik",{"stahl":0.5},15,profil=MB,rname="Pflüge und Sämaschinen bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.08,0.5)),
 E("maehdrescher","Mähdrescher",40000.0,"stueck",8000,None,R("maehdrescherwerk","Mähdrescherwerk",{"motor":1,"stahl":6,"getriebe":2},400,profil=MB,tech="maehdrescher",rname="Mähdrescher bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.003,0.5,{1938:1.0,1960:3.0,2026:3.0})),
 E("textilmaschine","Textilmaschine",20000.0,"stueck",4000,None,R("textilmaschinenfabrik","Textilmaschinenfabrik",{"stahl":2,"getriebe":1,"elektromotor":1},250,profil=MB,rname="Spinn- und Webmaschinen bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.004,1.0)),
 E("druckmaschine","Druckmaschine",60000.0,"stueck",15000,None,R("druckmaschinenfabrik","Druckmaschinenfabrik",{"stahl":8,"getriebe":3,"elektromotor":2},700,profil=MB,rname="Druckmaschinen bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.002,1.0)),
 E("verpackungsmaschine","Verpackungsmaschine",40000.0,"stueck",2000,None,R("verpackungsmaschinenwerk","Verpackungsmaschinenwerk",{"stahl":1.5,"elektromotor":2,"getriebe":1},400,profil=MB,tech="verpackungsautomat",rname="Verpackungsautomaten bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.002,1.0,{1950:1.0,1980:3.0,2026:5.0})),
 E("bergbaumaschine","Bergbaumaschine",50000.0,"stueck",15000,None,R("bergbaumaschinenfabrik","Bergbaumaschinenfabrik",{"stahl":5,"elektromotor":2,"getriebe":2},500,profil=MB,rname="Förder- und Abbaumaschinen bauen"), wg="maschinen", branche="maschinenbau", verw="industrie", staat=(0.002,1.0)),
]

# Machines are built from components (gears, motors, engines): five levels from the ore
# (Lastenheft §17.2, `sehr_komplex`).
for _p in P2B:
    if _p.get("wg") == "maschinen" and _p.get("art") == "endprodukt":
        _p["komplex"] = True
