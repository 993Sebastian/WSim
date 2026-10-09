# Spezifikation P4b – Elektronik, IT und Kommunikation (Felder wie in p1/spec.py und
# p3a/spec.py). Preise zum Erfindungsjahr, USD Kaufkraft 2026. Staatsnachfrage über
# st(‰ des BIP, Preis): Post, Netzbetreiber, Banken, Behörden und Gewerbe kaufen
# Vermittlungstechnik, Rechner, Kopierer und Steuerungen als Staat und Wirtschaft.

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
P4B = [
 # Bauteile
 E("leiterplatte","Leiterplatte",15,"stueck",0.1,None,R("leiterplattenwerk","Leiterplattenwerk",{"kupfer":0.0002,"glas":0.0003,"benzol":0.0002},0.08,profil=CH,tech="leiterplatte",umsatz=40),
   art="komponente", verw="industrie"),
 E("lautsprecher","Lautsprecher",8,"stueck",0.5,None,R("lautsprecherwerk","Lautsprecherwerk",{"stahl":0.0005,"kupferdraht":0.00005,"zellstoff":0.0001},0.05,tech="dynamischer_lautsprecher",umsatz=20),
   art="komponente", verw="industrie"),
 E("glasfaser","Lichtwellenleiter",2000,"km",20,None,R("glasfaserwerk","Glasfaserwerk",{"quarz":0.02,"polyethylen":0.02},8,profil=CH,tech="lichtwellenleiter",umsatz=40,rname="Glasfasern ziehen"),
   art="halbzeug", verw="industrie"),
 E("festplatte","Festplattenlaufwerk",300,"stueck",0.6,None,R("festplattenwerk","Festplattenwerk",{"aluminium":0.0003,"kleinmotor":1,"mikrochip":10,"glas":0.0001},1.5,tech="festplatte",umsatz=60),
   art="komponente", verw="industrie"),
 E("flashspeicher","Flash-Speicher",10,"stueck",0.002,None,R("flashwerk","Flash-Speicherwerk",{"reinstsilizium":0.003,"kupfer":0.00001},0.04,tech="flashspeicher",umsatz=60),
   art="komponente", verw="industrie"),
 # Kommunikation
 E("telefonapparat","Telefonapparat",150,"stueck",2,("g",15,0.9,4000,1.2,1.2,GB),
   R("telefonwerk","Telefonapparatewerk",{"kupferdraht":0.0003,"schnittholz":0.001,"stahl":0.0005,"messing":0.0002},1.0,umsatz=20,rname="Telefonapparate bauen")),
 E("fernschreiber","Fernschreiber",3000,"stueck",40,None,
   R("fernschreiberwerk","Fernschreiberwerk",{"kleinmotor":2,"stahl":0.01,"kupferdraht":0.001,"radioroehre":5},15,tech="fernschreiber",umsatz=20),
   staat=st(0.02,3000,2.0,{1930:0.5,1960:1.0,1990:0.2,2026:0.01}), **INDUSTRIE),
 E("faxgeraet","Faxgerät",600,"stueck",6,("g",8,0.1,30000,1.3,1.2,GB),
   R("faxwerk","Faxgerätewerk",{"mikrochip":30,"kleinmotor":2,"kunststoffwaren":3,"leiterplatte":1},3,tech="telefax",umsatz=30)),
 E("anrufbeantworter","Anrufbeantworter",150,"stueck",1,("g",8,0.3,20000,1.3,1.2,GB),
   R("anrufbeantworterwerk","Anrufbeantworterwerk",{"mikrochip":10,"kleinmotor":1,"kunststoffwaren":0.8,"lautsprecher":1},0.8,tech="anrufbeantworter",umsatz=20)),
 E("schnurlostelefon","Schnurlostelefon",80,"stueck",0.4,("g",6,0.6,15000,1.2,1.2,GB),
   R("schnurlostelefonwerk","Schnurlostelefonwerk",{"mikrochip":15,"kunststoffwaren":0.4,"leiterplatte":1,"lautsprecher":1},0.4,tech="schnurlostelefon",umsatz=30)),
 E("vermittlungsstelle","Fernsprechvermittlung",2000000,"stueck",30000,None,
   R("vermittlungswerk","Vermittlungstechnikwerk",{"stahl":20,"kupferdraht":5,"elektromotor":10,"kabel":2},9000,tech="waehlamt",umsatz=100,tage=30),
   staat=st(0.05,2000000,1.0,{1910:0.3,1950:1.0,2026:0.5}), **INDUSTRIE),
 E("mobilfunkbasisstation","Mobilfunk-Basisstation",150000,"stueck",800,None,
   R("basisstationswerk","Basisstationswerk",{"mikrochip":5000,"stahl":3,"kabel":0.3,"leiterplatte":20},600,tech="mobilfunk",umsatz=100),
   staat=st(0.05,150000,1.0,{1992:0.3,2005:1.0,2026:1.5}), **INDUSTRIE),
 E("router","Internet-Router",100,"stueck",0.5,("g",5,0.7,15000,1.2,1.2,GB),
   R("routerwerk","Routerwerk",{"mikrochip":20,"leiterplatte":1,"kunststoffwaren":0.3},0.4,tech="dsl_router",umsatz=40)),
 E("glasfaserkabel","Glasfaserkabel",8000,"km",300,None,
   R("glasfaserkabelwerk","Glasfaserkabelwerk",{"glasfaser":1.0,"polyethylen":0.1,"stahl":0.05},30,profil=CH,tech="lichtwellenleiter",umsatz=60),
   staat=st(0.1,8000,1.0,{1980:0.2,2000:1.0,2026:2.0}), **INDUSTRIE),
 E("funkgeraet","Sprechfunkgerät",800,"stueck",3,None,
   R("funkgeraetewerk","Funkgerätewerk",{"radioroehre":6,"kupferdraht":0.0005,"stahl":0.002,"trockenbatterie":4},4,tech="sprechfunk",umsatz=20),
   staat=st(0.01,800,4.0,{1940:1.0,2026:1.0}), **INDUSTRIE),
 E("pager","Funkmeldeempfänger (Pager)",100,"stueck",0.1,("g",3,0.1,20000,1.2,1.2,GB),
   R("pagerwerk","Pagerwerk",{"mikrochip":5,"kunststoffwaren":0.1,"trockenbatterie":1},0.4,tech="funkruf",umsatz=10)),
 E("smartwatch","Smartwatch",300,"stueck",0.05,("g",3,0.3,30000,1.4,1.3,GB),
   R("smartwatchwerk","Smartwatchwerk",{"mikrochip":30,"lithium_ionen_akku":1,"lcd_panel":0.002,"glas":0.00002},1.2,tech="smartwatch",umsatz=60)),
 E("fitnessarmband","Fitnessarmband",80,"stueck",0.03,("g",3,0.2,20000,1.3,1.2,GB),
   R("fitnessarmbandwerk","Fitnessarmbandwerk",{"mikrochip":8,"lithium_ionen_akku":1,"kunststoffwaren":0.03},0.3,tech="fitnessarmband",umsatz=30)),
 E("e_book_reader","E-Book-Lesegerät",120,"stueck",0.2,("g",5,0.15,25000,1.3,1.2,GB),
   R("ebookwerk","E-Book-Werk",{"mikrochip":20,"lithium_ionen_akku":3,"kunststoffwaren":0.2,"glas":0.0001},0.5,tech="elektronisches_papier",umsatz=30)),
 E("kopfhoerer","Kopfhörer",30,"stueck",0.3,("g",4,0.6,6000,1.2,1.0,GB),
   R("kopfhoererwerk","Kopfhörerwerk",{"lautsprecher":2,"kupferdraht":0.00002,"stahl":0.0001},0.15,tech="dynamischer_lautsprecher",umsatz=20)),
 E("funkkopfhoerer","Funkkopfhörer",150,"stueck",0.05,("g",3,0.4,25000,1.3,1.2,GB),
   R("funkkopfhoererwerk","Funkkopfhörerwerk",{"lautsprecher":2,"mikrochip":10,"lithium_ionen_akku":2,"kunststoffwaren":0.05},0.6,tech="funkkopfhoerer",umsatz=40)),
 E("smart_speaker","Sprachassistent-Lautsprecher",80,"stueck",0.8,("g",4,0.3,25000,1.3,1.2,GB),
   R("smartspeakerwerk","Smartspeakerwerk",{"lautsprecher":2,"mikrochip":15,"kunststoffwaren":0.6,"leiterplatte":1},0.3,tech="sprachassistent",umsatz=30)),
 # Unterhaltung
 E("grammophon","Grammophon",400,"stueck",8,("g",15,0.3,8000,1.4,1.4,GB),
   R("grammophonwerk","Grammophonwerk",{"schnittholz":0.005,"messing":0.001,"stahl":0.002},3,profil=MB,umsatz=10,rname="Grammophone bauen")),
 E("plattenspieler","Elektrischer Plattenspieler",200,"stueck",6,("g",12,0.5,10000,1.3,1.3,GB),
   R("plattenspielerwerk","Plattenspielerwerk",{"kleinmotor":1,"lautsprecher":2,"schnittholz":0.003,"radioroehre":3},1.2,tech="elektrischer_plattenspieler",umsatz=20),
   ersetzt=["grammophon"]),
 E("tonbandgeraet","Tonbandgerät",600,"stueck",12,("g",12,0.15,15000,1.4,1.4,GB),
   R("tonbandwerk","Tonbandgerätewerk",{"kleinmotor":2,"radioroehre":5,"lautsprecher":1,"stahl":0.003},3,tech="tonband",umsatz=20)),
 E("kassettenrekorder","Kassettenrekorder",120,"stueck",2,("g",8,0.5,10000,1.2,1.2,GB),
   R("kassettenwerk","Kassettenrekorderwerk",{"kleinmotor":1,"transistor":20,"lautsprecher":1,"kunststoffwaren":0.8},0.6,tech="kompaktkassette",umsatz=30),
   ersetzt=["tonbandgeraet"]),
 E("walkman","Tragbarer Kassettenspieler",100,"stueck",0.3,("g",4,0.4,12000,1.3,1.2,GB),
   R("walkmanwerk","Werk für tragbare Kassettenspieler",{"kleinmotor":1,"transistor":30,"kunststoffwaren":0.2,"trockenbatterie":2},0.5,tech="tragbarer_kassettenspieler",umsatz=30)),
 E("mp3_spieler","MP3-Spieler",150,"stueck",0.1,("g",4,0.4,15000,1.3,1.2,GB),
   R("mp3werk","MP3-Spieler-Werk",{"flashspeicher":2,"mikrochip":10,"lithium_ionen_akku":2,"kunststoffwaren":0.1},0.5,tech="mp3_spieler",umsatz=40),
   ersetzt=["walkman"]),
 E("stereoanlage","Stereoanlage",800,"stueck",20,("g",12,0.5,15000,1.3,1.3,GB),
   R("hifiwerk","Hi-Fi-Werk",{"lautsprecher":4,"transistor":100,"schnittholz":0.01,"kleinmotor":2},4,tech="hifi_stereo",umsatz=30)),
 E("autoradio","Autoradio",150,"stueck",1.5,("e","automobil",0.1,8000,1.0,1.0,GB),
   R("autoradiowerk","Autoradiowerk",{"radioroehre":4,"lautsprecher":1,"stahl":0.001,"kupferdraht":0.0002},0.8,tech="autoradio",umsatz=20)),
 E("spielkonsole","Spielkonsole",300,"stueck",3,("g",6,0.5,20000,1.3,1.3,GB),
   R("konsolenwerk","Spielkonsolenwerk",{"mikrochip":40,"leiterplatte":2,"kunststoffwaren":1.5},1.2,tech="videospielkonsole",umsatz=60)),
 E("handheld_konsole","Tragbare Spielkonsole",150,"stueck",0.4,("g",4,0.3,15000,1.3,1.3,GB),
   R("handheldwerk","Handheld-Werk",{"mikrochip":20,"leiterplatte":1,"kunststoffwaren":0.3,"trockenbatterie":4},0.6,tech="handheld_konsole",umsatz=40)),
 E("videokamera","Videokamera",1500,"stueck",1.5,("g",6,0.15,25000,1.4,1.3,GB),
   R("videokamerawerk","Videokamerawerk",{"kleinmotor":2,"mikrochip":30,"glas":0.0005,"kunststoffwaren":1},6,tech="camcorder",umsatz=30)),
 E("bluray_player","Blu-ray-Spieler",300,"stueck",3,("g",6,0.3,20000,1.2,1.2,GB),
   R("bluraywerk","Blu-ray-Werk",{"mikrochip":30,"kleinmotor":2,"kunststoffwaren":1.5,"leiterplatte":1},1.2,tech="blu_ray",umsatz=30)),
 E("beamer","Videoprojektor",2500,"stueck",4,("g",6,0.05,40000,1.4,1.4,LX),
   R("beamerwerk","Projektorenwerk",{"lcd_panel":0.005,"halogenlampe":1,"mikrochip":30,"kunststoffwaren":2},10,tech="videoprojektor",umsatz=30),
   staat=st(0.005,2500,1.0,{1992:0.5,2010:1.0,2026:0.7})),
 E("jukebox","Musikautomat (Jukebox)",3000,"stueck",150,None,
   R("musikautomatenwerk","Musikautomatenwerk",{"kleinmotor":2,"radioroehre":6,"lautsprecher":2,"schnittholz":0.05},15,tech="musikautomat",umsatz=10),
   staat=st(0.002,3000,1.0,{1927:1.0,1960:1.0,1990:0.2,2026:0.05}), **INDUSTRIE),
 # Rechner und Bürotechnik
 E("grossrechner","Großrechner",3000000,"stueck",5000,None,
   R("grossrechnerwerk","Großrechnerwerk",{"mikrochip":20000,"leiterplatte":200,"stahl":5,"kupferdraht":1},12000,tech="grossrechner",umsatz=150,tage=30),
   staat=st(0.05,3000000,1.0,{1964:1.0,1990:0.6,2026:0.2}), **INDUSTRIE),
 E("server","Server",8000,"stueck",30,None,
   R("serverwerk","Serverwerk",{"mikrochip":200,"festplatte":4,"leiterplatte":4,"stahl":0.01},30,tech="server",umsatz=150),
   staat=st(0.3,8000,1.0,{1995:0.3,2010:1.0,2026:2.0}), **INDUSTRIE),
 E("drucker","Drucker",300,"stueck",8,("g",6,0.5,20000,1.2,1.2,GB),
   R("druckerwerk","Druckerwerk",{"kleinmotor":3,"mikrochip":10,"kunststoffwaren":3,"leiterplatte":1},1.2,tech="laserdrucker",umsatz=60)),
 E("scanner","Flachbettscanner",200,"stueck",4,("g",6,0.2,25000,1.2,1.2,GB),
   R("scannerwerk","Scannerwerk",{"kleinmotor":1,"mikrochip":10,"glas":0.002,"kunststoffwaren":2},0.8,tech="flachbettscanner",umsatz=20)),
 E("usb_stick","USB-Stick",15,"stueck",0.01,("g",3,0.8,15000,1.0,1.0,GB),
   R("usbwerk","USB-Stick-Werk",{"flashspeicher":1,"kunststoffwaren":0.01},0.05,tech="usb_speicher",umsatz=30)),
 E("externe_festplatte","Externe Festplatte",100,"stueck",0.3,("g",5,0.4,20000,1.2,1.2,GB),
   R("externe_festplattenwerk","Werk für externe Festplatten",{"festplatte":1,"leiterplatte":1,"kunststoffwaren":0.2},0.3,tech="externe_festplatte",umsatz=30)),
 E("monitor","Computerbildschirm",300,"stueck",10,("g",7,0.5,20000,1.2,1.2,GB),
   R("monitorwerk","Bildschirmwerk",{"bildroehre":1,"leiterplatte":1,"kunststoffwaren":3},1.2,tech="computermonitor",umsatz=60),
   staat=st(0.02,300,1.0,{1981:0.3,2000:1.0,2026:0.6})),
 E("lochkartenmaschine","Lochkarten-Tabelliermaschine",30000,"stueck",400,None,
   R("tabelliermaschinenwerk","Tabelliermaschinenwerk",{"stahl":0.5,"elektromotor":1,"messing":0.05,"kupferdraht":0.01},150,profil=MB,tech="tabelliermaschine",umsatz=20),
   wg="maschinen", branche="maschinenbau", staat=st(0.01,30000,1.0,{1906:0.2,1930:1.0,1960:1.0,1975:0.1,2026:0.01}), **INDUSTRIE),
 E("rechenmaschine","Mechanische Rechenmaschine",1500,"stueck",12,None,
   R("rechenmaschinenwerk","Rechenmaschinenwerk",{"stahl":0.01,"messing":0.003},8,profil=MB,tech="sprossenradrechner",umsatz=10),
   wg="maschinen", branche="maschinenbau", staat=st(0.02,1500,1.0,{1905:0.3,1930:1.0,1970:0.3,2026:0.01}), **INDUSTRIE),
 E("kopierer","Fotokopierer",3000,"stueck",60,None,
   R("kopiererwerk","Kopiererwerk",{"kleinmotor":3,"glas":0.003,"stahl":0.02,"transistor":50},15,tech="xerografie",umsatz=40),
   staat=st(0.03,3000,1.0,{1959:0.3,1980:1.0,2026:0.6}), **INDUSTRIE),
 E("geldautomat","Geldautomat",30000,"stueck",800,None,
   R("geldautomatenwerk","Geldautomatenwerk",{"mikrochip":500,"stahl":0.3,"kleinmotor":4,"leiterplatte":10},120,tech="geldautomat",umsatz=40),
   staat=st(0.01,30000,1.0,{1967:0.2,1985:1.0,2026:0.6}), **INDUSTRIE),
 E("sps","Speicherprogrammierbare Steuerung",3000,"stueck",3,None,
   R("spswerk","Steuerungswerk",{"mikrochip":100,"leiterplatte":3,"kupferdraht":0.0005,"stahl":0.002},12,tech="sps",umsatz=60),
   staat=st(0.05,3000,1.0,{1969:0.2,1985:1.0,2026:1.5}), **INDUSTRIE),
 E("ueberwachungskamera","Überwachungskamera",200,"stueck",0.8,("g",6,0.2,25000,1.2,1.2,GB),
   R("ueberwachungskamerawerk","Überwachungskamerawerk",{"mikrochip":20,"glas":0.0002,"kunststoffwaren":0.3,"kabel":0.0002},0.8,tech="videoueberwachung",umsatz=30),
   staat=st(0.01,200,1.0,{1973:0.3,2000:1.0,2026:2.0})),
 E("alarmanlage","Alarmanlage",500,"stueck",3,("g",12,0.2,25000,1.2,1.2,GB),
   R("alarmanlagenwerk","Alarmanlagenwerk",{"mikrochip":20,"kabel":0.002,"kunststoffwaren":1,"lautsprecher":1},2,tech="einbruchmeldeanlage",umsatz=20)),
]

EINFACH = {"leiterplatte", "lautsprecher", "glasfaser", "flashspeicher", "telefonapparat", "grammophon", "rechenmaschine", "fernschreiber"}
for _p in P4B:
    # Electronics from chips, boards and cells reach five or six levels (§17.2).
    if _p["id"] not in EINFACH:
        _p["komplex"] = True
