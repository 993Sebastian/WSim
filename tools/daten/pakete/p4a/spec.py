# Spezifikation P4a – Elektrotechnik und Haushaltsgeräte (Felder wie in p1/spec.py und
# p3a/spec.py). Preise zum Erfindungsjahr, USD Kaufkraft 2026. Staatsnachfrage über
# st(‰ des BIP, Preis): Stromversorger, Bauwirtschaft und Gewerbe kaufen Kabel, Zähler,
# Transformatoren, Generatoren und Schaltanlagen als Staat und Wirtschaft.

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
P4A = [
 # Werkstoffe und Bauteile
 P(id="nickelerz", name="Nickelerz (Konzentrat)", art="rohstoff", branche="bergbau", einheit="t", wg="erze", tk="schuettgut", preis=1500, verw="industrie",
   rez=dict(anlage=("nickelgrube","Nickelgrube","foerderstaette",3), h=25, profil=FARM, abbau=True, rname="Nickelerz fördern"),
   index={1900:1.0,1930:3.0,1950:5.0,1970:12.0,1990:16.0,2010:25.0,2026:40.0}, pacht=0.3,
   dep=[("nickel_can","Sudbury","CAN",8e3,0.8,2e6),("nickel_ncl","Neukaledonien","XOZ",6e3,0.9,2e6),("nickel_rus","Norilsk und Petsamo","RUS",3e3,1.0,3e6),
        ("nickel_aus","Kambalda","AUS",2e3,0.9,1.5e6),("nickel_idn","Sulawesi","IDN",2e3,0.9,3e6),("nickel_phl","Nickel Philippinen","PHL",1e3,0.9,1e6),
        ("nickel_cub","Moa","CUB",1e3,1.0,8e5),("nickel_zaf","Bushveld","ZAF",1e3,1.0,8e5),("nickel_bra","Nickel Goiás","BRA",1e3,1.0,8e5),("nickel_fin","Nickel Finnland","FIN",1e3,1.0,5e5)]),
 E("nickel","Nickel",20000,"t",None,None,R("nickelhuette","Nickelhütte",{"nickelerz":4.0,"kohle":2.0},40,profil=ME,umsatz=20,rname="Nickel schmelzen und raffinieren"),
   # Stainless and armour steel, coins and plating: bought by state and industry.
   art="halbzeug", verw="industrie", wg="nichteisenmetalle", branche="metallurgie", staat=st(0.05,20000,3.0,{1900:0.5,1930:1.0,1970:2.0,2026:3.0})),
 E("heizleiter","Heizleiter (Chromnickeldraht)",30000,"t",None,None,R("heizleiterwerk","Heizleiterwerk",{"nickel":0.8,"stahl":0.3},60,profil=ME,tech="heizleiter",umsatz=10,rname="Chromnickeldraht ziehen"),
   art="halbzeug", verw="industrie", wg="nichteisenmetalle", branche="metallurgie"),
 E("kabel","Elektrokabel",6000,"t",None,None,R("kabelwerk","Kabelwerk",{"kupferdraht":0.5,"gummi":0.03,"blei":0.2},40,umsatz=30,rname="Kabel isolieren und ummanteln"),
   art="halbzeug", verw="industrie", staat=st(0.3,6000,1.5,{1900:0.3,1930:1.0,1970:2.0,2026:2.0})),
 E("kaelteaggregat","Kälteaggregat",300,"stueck",15,None,R("kaelteaggregatwerk","Kälteaggregatwerk",{"kleinmotor":1,"kupfer":0.005,"stahl":0.01},2,profil=MB,tech="kompressionskuehlschrank",umsatz=20),
   art="komponente", verw="industrie"),
 E("trockenbatterie","Trockenbatterie",2,"stueck",0.1,("v",3,3000,1.0,1.0,GB),
   R("batteriefabrik","Batteriefabrik",{"zink":0.00003,"kohle":0.00003},0.01,profil=CH,umsatz=10,rname="Zink-Kohle-Zellen füllen")),
 # Haushaltsgeräte
 E("buegeleisen","Elektrisches Bügeleisen",60,"stueck",1.5,("g",10,0.9,4000,1.2,1.0,GB),
   R("buegeleisenwerk","Bügeleisenwerk",{"heizleiter":0.0002,"stahl":0.0015,"kupferdraht":0.0002},0.5,tech="elektrisches_buegeleisen",umsatz=20)),
 E("toaster","Toaster",50,"stueck",1.5,("g",10,0.8,6000,1.2,1.0,GB),
   R("toasterwerk","Toasterwerk",{"heizleiter":0.0001,"blech":0.001,"kupferdraht":0.0001},0.4,tech="toaster",umsatz=20)),
 E("wasserkocher","Wasserkocher",40,"stueck",1.0,("g",8,0.8,8000,1.2,1.0,GB),
   R("wasserkocherwerk","Wasserkocherwerk",{"heizleiter":0.0001,"blech":0.0008,"kupferdraht":0.0001},0.3,tech="wasserkocher",umsatz=20)),
 E("kaffeemaschine","Kaffeemaschine",80,"stueck",2.0,("g",8,0.7,12000,1.2,1.0,GB),
   R("kaffeemaschinenwerk","Kaffeemaschinenwerk",{"heizleiter":0.0001,"kunststoffwaren":0.8,"glas":0.0005,"kupferdraht":0.0001},0.5,tech="kaffeemaschine",umsatz=30)),
 E("elektroherd","Elektroherd",900,"stueck",50,("g",15,0.9,8000,1.2,1.0,GB),
   R("elektroherdwerk","Elektroherdwerk",{"heizleiter":0.002,"blech":0.04,"stahl":0.01,"kupferdraht":0.001},6,tech="elektroherd",umsatz=40)),
 E("induktionskochfeld","Induktionskochfeld",700,"stueck",12,("g",12,0.4,25000,1.3,1.2,GB),
   R("kochfeldwerk","Kochfeldwerk",{"kupferdraht":0.002,"glas":0.006,"mikrochip":20,"blech":0.003},3,tech="induktionskochfeld",umsatz=30)),
 E("gefrierschrank","Gefrierschrank",700,"stueck",50,("g",15,0.6,15000,1.3,1.2,GB),
   R("gefriergeraetewerk","Gefriergerätewerk",{"kaelteaggregat":1,"blech":0.04,"stahl":0.01,"kunststoffwaren":3},5,tech="gefriergeraet",umsatz=40)),
 E("geschirrspueler","Geschirrspüler",800,"stueck",45,("g",12,0.7,20000,1.4,1.2,GB),
   R("geschirrspuelerwerk","Geschirrspülerwerk",{"kleinmotor":2,"blech":0.03,"heizleiter":0.0003,"kunststoffwaren":3},5,tech="geschirrspueler",umsatz=40)),
 E("waeschetrockner","Wäschetrockner",600,"stueck",40,("g",12,0.5,20000,1.4,1.2,GB),
   R("trocknerwerk","Wäschetrocknerwerk",{"kleinmotor":1,"blech":0.03,"heizleiter":0.0005,"kunststoffwaren":2},4,tech="waeschetrockner",umsatz=40)),
 E("dunstabzugshaube","Dunstabzugshaube",300,"stueck",12,("g",15,0.5,18000,1.2,1.0,GB),
   R("dunstabzugwerk","Dunstabzugswerk",{"kleinmotor":1,"blech":0.01,"gluehlampe":2},2,tech="dunstabzug",umsatz=20)),
 E("mixer","Standmixer",60,"stueck",2.5,("g",8,0.6,10000,1.2,1.0,GB),
   R("mixerwerk","Mixerwerk",{"kleinmotor":1,"glas":0.001,"blech":0.0005},0.4,tech="standmixer",umsatz=20)),
 E("kuechenmaschine","Küchenmaschine",300,"stueck",8,("g",12,0.4,15000,1.3,1.2,GB),
   R("kuechenmaschinenwerk","Küchenmaschinenwerk",{"kleinmotor":1,"stahl":0.004,"blech":0.002},2,tech="kuechenmaschine",umsatz=20)),
 E("fritteuse","Fritteuse",60,"stueck",3,("g",8,0.4,15000,1.2,1.0,GB),
   R("fritteusenwerk","Fritteusenwerk",{"heizleiter":0.0001,"blech":0.002,"kunststoffwaren":0.5},0.4,tech="fritteuse",umsatz=20)),
 E("elektrogrill","Elektrogrill",80,"stueck",4,("g",8,0.4,12000,1.2,1.0,GB),
   R("elektrogrillwerk","Elektrogrillwerk",{"heizleiter":0.0002,"blech":0.003,"stahl":0.001},0.5,tech="elektrogrill",umsatz=20)),
 E("brotbackautomat","Brotbackautomat",120,"stueck",6,("g",8,0.15,20000,1.3,1.2,GB),
   R("brotbackautomatenwerk","Brotbackautomatenwerk",{"heizleiter":0.0001,"kleinmotor":1,"blech":0.003,"mikrochip":5},0.8,tech="brotbackautomat",umsatz=20)),
 E("elektrorasierer","Elektrorasierer",60,"stueck",0.3,("g",6,0.6,8000,1.2,1.0,GB),
   R("rasiererwerk","Rasiererwerk",{"kleinmotor":1,"stahl":0.0002,"bakelit":0.0003},0.4,tech="trockenrasierer",umsatz=20)),
 E("haartrockner","Haartrockner",40,"stueck",0.6,("g",6,0.8,7000,1.2,1.0,GB),
   R("haartrocknerwerk","Haartrocknerwerk",{"kleinmotor":1,"heizleiter":0.00005,"bakelit":0.0004},0.3,tech="haartrockner",umsatz=20)),
 E("elektrische_zahnbuerste","Elektrische Zahnbürste",40,"stueck",0.2,("g",4,0.5,15000,1.3,1.2,GB),
   R("zahnbuerstenwerk","Zahnbürstenwerk",{"kleinmotor":1,"kunststoffwaren":0.1,"kupferdraht":0.00002},0.2,tech="elektrische_zahnbuerste",umsatz=20)),
 E("ventilator","Ventilator",50,"stueck",3,("g",10,0.6,6000,1.2,1.0,GB),
   R("ventilatorwerk","Ventilatorwerk",{"kleinmotor":1,"blech":0.002,"kupferdraht":0.0001},0.4,tech="tischventilator",umsatz=20)),
 E("heizluefter","Heizlüfter",50,"stueck",2,("g",10,0.4,8000,1.2,1.0,GB),
   R("heizluefterwerk","Heizlüfterwerk",{"kleinmotor":1,"heizleiter":0.0001,"blech":0.0015},0.4,tech="heizluefter",umsatz=20)),
 E("heizdecke","Heizdecke",50,"stueck",1.5,("g",8,0.2,8000,1.2,1.0,GB),
   R("heizdeckenwerk","Heizdeckenwerk",{"heizleiter":0.00005,"stoff":0.001,"kupferdraht":0.0001},0.4,profil=TX,tech="heizdecke",umsatz=10)),
 E("nachtspeicherofen","Nachtspeicherofen",1200,"stueck",120,("g",25,0.15,15000,1.3,1.0,GB),
   R("speicherofenwerk","Speicherofenwerk",{"heizleiter":0.002,"blech":0.02,"mauerziegel":60},5,tech="nachtspeicherheizung",umsatz=20)),
 E("klimageraet","Klimagerät",900,"stueck",40,("g",12,0.5,20000,1.5,1.5,GB),
   R("klimageraetewerk","Klimagerätewerk",{"kaelteaggregat":1,"kleinmotor":1,"blech":0.02,"kunststoffwaren":2},5,tech="raumklimageraet",umsatz=60)),
 E("warmwasserspeicher","Warmwasserspeicher",400,"stueck",40,("g",15,0.6,10000,1.2,1.0,GB),
   R("boilerwerk","Warmwasserspeicherwerk",{"heizleiter":0.0005,"blech":0.03,"kupferdraht":0.0005},3,tech="warmwasserspeicher",umsatz=20)),
 E("luftreiniger","Luftreiniger",200,"stueck",6,("g",8,0.15,25000,1.3,1.2,GB),
   R("luftreinigerwerk","Luftreinigerwerk",{"kleinmotor":1,"kunststoffwaren":2,"mikrochip":5,"zellstoff":0.0005},1,tech="luftreiniger",umsatz=20)),
 E("saugroboter","Saugroboter",300,"stueck",3,("g",6,0.3,25000,1.4,1.3,GB),
   R("saugroboterwerk","Saugroboterwerk",{"kleinmotor":3,"mikrochip":40,"lithium_ionen_akku":4,"kunststoffwaren":1.5},1.5,tech="saugroboter",umsatz=40)),
 # Licht
 E("leuchte","Wohnraumleuchte",80,"stueck",2,("g",15,0.9,4000,1.1,1.0,GB),
   R("leuchtenwerk","Leuchtenwerk",{"messing":0.001,"glas":0.0005,"kupferdraht":0.0001},0.6,tech="elektrische_beleuchtung",umsatz=20)),
 E("taschenlampe","Taschenlampe",15,"stueck",0.3,("g",5,0.7,3000,1.2,1.0,GB),
   R("taschenlampenwerk","Taschenlampenwerk",{"blech":0.0002,"gluehlampe":1,"messing":0.00005},0.1,tech="taschenlampe",umsatz=10)),
 E("lichterkette","Lichterkette",30,"stueck",0.5,("g",6,0.5,6000,1.2,1.0,GB),
   R("lichterkettenwerk","Lichterkettenwerk",{"gluehlampe":20,"kupferdraht":0.0002},0.3,tech="lichterkette",umsatz=10)),
 E("leuchtstofflampe","Leuchtstofflampe",8,"stueck",0.2,("v",0.1,6000,1.0,1.0,GB),
   R("leuchtstofflampenwerk","Leuchtstofflampenwerk",{"glas":0.0002,"kupferdraht":0.00001,"zink":0.000005},0.04,tech="leuchtstofflampe",umsatz=20),
   staat=st(0.05,8,1.0,{1938:0.3,1960:1.0,2010:0.5,2026:0.2})),
 E("halogenlampe","Halogenlampe",5,"stueck",0.03,("v",0.2,10000,1.0,1.0,GB),
   R("halogenlampenwerk","Halogenlampenwerk",{"glas":0.00003,"kupferdraht":0.000005},0.02,tech="halogenlampe",umsatz=10)),
 E("energiesparlampe","Energiesparlampe",8,"stueck",0.1,("v",0.3,12000,1.0,1.0,GB),
   R("energiesparlampenwerk","Energiesparlampenwerk",{"glas":0.00008,"kupferdraht":0.00001,"transistor":2},0.04,tech="kompaktleuchtstofflampe",umsatz=20)),
 # Elektrotechnik für Versorger, Bau und Gewerbe
 E("elektroinstallation","Schalter und Steckdosen",8,"stueck",0.1,None,
   R("installationswerk","Installationsmaterialwerk",{"messing":0.00003,"bakelit":0.00006,"kupferdraht":0.00001},0.04,tech="schutzkontakt",umsatz=20),
   staat=st(0.15,8,1.0,{1926:0.5,1960:1.0,2026:1.0}), **INDUSTRIE),
 E("stromzaehler","Stromzähler",120,"stueck",3,None,
   R("zaehlerwerk","Zählerwerk",{"kupferdraht":0.0003,"blech":0.001,"glas":0.0002,"messing":0.0002},0.8,umsatz=20,rname="Stromzähler bauen"),
   staat=st(0.05,120,1.0,{1900:0.3,1930:1.0,2026:1.0}), **INDUSTRIE),
 E("transformator","Leistungstransformator",150000,"stueck",20000,None,
   R("transformatorenwerk","Transformatorenwerk",{"blech":12,"kupferdraht":4,"stahl":3},800,tech="grosstransformator",umsatz=60,rname="Transformatoren wickeln"),
   staat=st(0.1,150000,1.5,{1904:0.3,1930:1.0,1970:1.5,2026:1.5}), **INDUSTRIE),
 E("kraftwerksgenerator","Kraftwerksgenerator",2000000,"stueck",120000,None,
   R("generatorwerk","Generatorwerk",{"stahl":80,"kupferdraht":15,"blech":20},10000,tech="turbogenerator",umsatz=150,tage=30,rname="Generatoren bauen"),
   staat=st(0.05,2000000,1.5,{1903:0.3,1930:1.0,1970:1.5,2026:1.0}), **INDUSTRIE),
 E("schaltanlage","Hochspannungsschaltanlage",800000,"stueck",30000,None,
   R("schaltanlagenwerk","Schaltanlagenwerk",{"stahl":15,"kupfer":3,"kabel":2,"glas":0.5},4000,tech="hochspannungstechnik",umsatz=80),
   staat=st(0.05,800000,1.5,{1912:0.3,1950:1.0,2026:1.2}), **INDUSTRIE),
 E("schweissgeraet","Elektroschweißgerät",2500,"stueck",60,None,
   R("schweissgeraetewerk","Schweißgerätewerk",{"blech":0.02,"kupferdraht":0.01,"kabel":0.005},12,tech="lichtbogenschweissen",umsatz=20),
   staat=st(0.03,2500,2.0,{1920:0.5,1950:1.0,2026:1.0}), **INDUSTRIE),
 E("notstromaggregat","Notstromaggregat",30000,"stueck",1500,None,
   R("aggregatwerk","Aggregatwerk",{"dieselmotor":1,"kupferdraht":0.1,"stahl":0.4},120,profil=MB,tech="notstromaggregat",umsatz=40),
   staat=st(0.02,30000,2.0,{1930:0.5,1970:1.0,2026:1.2}), **INDUSTRIE),
 E("gewerbekuehlung","Gewerbekühlanlage",15000,"stueck",600,None,
   R("gewerbekuehlwerk","Gewerbekühlwerk",{"kaelteaggregat":4,"blech":0.3,"stahl":0.1,"kunststoffwaren":20},60,tech="gefriergeraet",umsatz=40),
   staat=st(0.03,15000,1.0,{1940:0.5,1970:1.0,2026:1.2}), **INDUSTRIE),
 E("rolltreppe","Rolltreppe",300000,"stueck",10000,None,
   R("rolltreppenwerk","Rolltreppenwerk",{"stahl":8,"elektromotor":2,"gummi":0.3,"glas":0.5},1500,profil=MB,tech="rolltreppe",umsatz=40),
   wg="maschinen", branche="maschinenbau", staat=st(0.01,300000,0.5,{1911:0.1,1950:1.0,2026:1.5}), **INDUSTRIE),
 E("akkuschrauber","Akkuschrauber",80,"stueck",1.2,("g",8,0.3,15000,1.3,1.2,GB),
   R("akkuschrauberwerk","Akkuschrauberwerk",{"kleinmotor":1,"lithium_ionen_akku":5,"kunststoffwaren":0.4,"stahl":0.0003},0.5,tech="akkuschrauber",umsatz=30)),
]

KOMPLEX = {"dunstabzugshaube", "fritteuse", "taschenlampe", "lichterkette", "schweissgeraet", "saugroboter", "induktionskochfeld", "brotbackautomat", "luftreiniger", "akkuschrauber", "energiesparlampe", "gefrierschrank", "klimageraet", "gewerbekuehlung", "schaltanlage", "kaffeemaschine", "geschirrspueler", "waeschetrockner", "elektrische_zahnbuerste", "notstromaggregat", "rolltreppe"}
for _p in P4A:
    # Appliances from chips, cells, plastics or compressors reach five levels (§17.2).
    if _p["id"] in KOMPLEX:
        _p["komplex"] = True
