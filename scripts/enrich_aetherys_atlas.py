#!/usr/bin/env python3
"""Add fully editable, scale-independent vector terrain detail to Aetherys.
Uses existing biome polygons, does not embed raster images, and leaves all existing geometry intact.
Idempotent: regenerated layers replace previous generated layers.
"""
import json, math, random
from pathlib import Path
PATH = Path(__file__).resolve().parents[1] / "maps" / "Aetherys World Map2.vectorcraft"
data=json.loads(PATH.read_text())
doc=data["document"]
rng=random.Random(20261106)
prefix="Atlasdetail · "
doc["layers"]=[l for l in doc["layers"] if not l["name"].startswith(prefix)]
biomes=next(l for l in doc["layers"] if l["name"].startswith("Biomzonen"))
uid=max([doc.get("next_id",0)]+[l["id"] for l in doc["layers"]])+1
def ident():
    global uid
    v=uid; uid+=1
    return v
def rgb(hexcolor):
    return dict(model="rgb",r=int(hexcolor[:2],16)/255,g=int(hexcolor[2:4],16)/255,b=int(hexcolor[4:],16)/255)
def subpath(coords):
    return dict(anchors=[dict(p=[round(x,3),round(y,3)]) for x,y in coords],closed=False)
buckets={k:[] for k in ("forest","forest_highlight","rocks","rock_highlight","snow","ice","dune","dune_shadow","grass")}
def inside(x,y,polys):
    for p in polys:
        odd=False
        for i in range(len(p)):
            ax,ay=p[i-1]; bx,by=p[i]
            if (ay>y)!=(by>y) and x<(bx-ax)*(y-ay)/(by-ay)+ax:
                odd=not odd
        if odd:return True
    return False
count=0
for obj in biomes["kind"]["children"]:
    nm=obj.get("name","").lower()
    polys=[[a["p"] for a in s["anchors"]] for s in obj["kind"]["path"]["subpaths"] if s.get("closed")]
    if not polys:continue
    points=[p for poly in polys for p in poly]
    left=min(p[0] for p in points); right=max(p[0] for p in points)
    top=min(p[1] for p in points); bottom=max(p[1] for p in points)
    snow=any(k in nm for k in ("schnee","eis","tundra"))
    mountain=any(k in nm for k in ("berg","hochland","hochebene","gebirge","schneefeld"))
    woods=any(k in nm for k in ("wald","wälder","regenwald","urwald"))
    dry=any(k in nm for k in ("wüste","trocken","steppe","gras","wiesen"))
    area=(right-left)*(bottom-top)
    target=min(720,max(30,round(area*(0.12 if woods or mountain else 0.085))))
    accepted=0
    for trial in range(target*8):
        if accepted>=target:break
        x=left+rng.random()*(right-left); y=top+rng.random()*(bottom-top)
        if not inside(x,y,polys):continue
        r=0.65+1.1*rng.random()
        if snow:
            a=r*(0.8+rng.random())
            coords=[(x-a,y+.15*r),(x,y-.5*r),(x+a,y+.05*r)]
            kind="ice"
            if rng.random()>.65:
                kind="snow"
        elif mountain:
            h=r*(1.7+rng.random()); w=r*(1.4+rng.random())
            coords=[(x-w,y+.3*h),(x,y-h),(x+w,y+.3*h)]
            kind="rocks"
            if inside(x-w,y+.3*h,polys) and inside(x+w,y+.3*h,polys):
                buckets["rock_highlight"].append(subpath([(x-.2*w,y-.58*h),(x,y-h),(x+.28*w,y-.45*h)]))
                if rng.random()>.35:
                    buckets["snow"].append(subpath([(x-.28*w,y-.37*h),(x,y-h),(x+.38*w,y-.3*h)]))
        elif woods:
            h=r*(1+rng.random());w=h*.5
            coords=[(x-w,y+.25*h),(x,y-h),(x+w,y+.25*h)]
            kind="forest"
            if rng.random()>.4:
                buckets["forest_highlight"].append(subpath([(x-.35*w,y-.2*h),(x,y-.75*h),(x+.18*w,y-.2*h)]))
        elif dry:
            w=1.8*r
            coords=[(x-w,y+.1*r),(x-.45*w,y-.33*r),(x+.3*w,y-.39*r),(x+w,y+.13*r)]
            kind="dune"
            if rng.random()>.65:
                buckets["dune_shadow"].append(subpath([(x-.6*w,y+.42*r),(x+.7*w,y+.39*r)]))
        else:
            coords=[(x-r,y+.1*r),(x,y-.45*r),(x+r,y+.12*r)]
            kind="grass"
        if all(inside(a,b,polys) for a,b in coords):
            buckets[kind].append(subpath(coords))
            accepted+=1
    count+=accepted
def obj(name,key,color,width):
    return dict(id=ident(),name=name,appearance=dict(items=[
        dict(kind="fill",paint=dict(type="none")),
        dict(kind="stroke",paint=dict(type="solid",color=rgb(color)),width=width)
    ]),kind=dict(type="path",path=dict(subpaths=buckets[key])))
def layer(title,specs):
    return dict(id=ident(),name=prefix+title,kind=dict(type="layer",color={"Preset":2},template=False,printable=True,children=[obj(*spec) for spec in specs]))
new=[
 layer("Wald und Baumkronen · editierbare Tuschestriche",[
    ("Nadelwald · dunkle Baumkronen","forest","184D32",0.32),
    ("Baumwipfel · Lichtseiten","forest_highlight","80A46A",0.19)]),
 layer("Gebirgszüge und Schnee · editierbare Grate",[
    ("Hauptfelsgrate und Bergsilhouetten","rocks","5E564A",0.48),
    ("Helle Felsschichten","rock_highlight","D5BE91",0.32),
    ("Schneegipfel und Firnkanten","snow","F7F8F0",0.43),
    ("Gletscher und Eisfeldschattierung","ice","7399AD",0.37)]),
 layer("Dünen Steppen Wiesen · editierbare Schraffuren",[
    ("Sandrücken und Dünenwellen","dune","AC854A",0.33),
    ("Schattige Dünentäler","dune_shadow","805F3B",0.24),
    ("Grasland · feine Vegetationsrisse","grass","637A48",0.24)])
]
idx=doc["layers"].index(biomes)
doc["layers"][idx+1:idx+1]=new
doc["next_id"]=uid
doc["title"]="Aetherys – detaillierter editierbarer Vektor-Atlas"
PATH.write_text(json.dumps(data,ensure_ascii=False,separators=(",",":")))
print(f"Added {sum(len(a) for a in buckets.values())} editable vector subpaths across 44 biome regions; {count} placement centers")
