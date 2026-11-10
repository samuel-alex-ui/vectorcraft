# AETH-08: Naht, Pole und Maße

**Status:** Radius und Messung sind in main. Der automatische Umbruch liegt zur Review vor.

**Reihenfolge:** 08 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Überprüfe Antimeridian für Linien und gefüllte Polygone inkl. Löcher sowie Polbereiche; ergänze kompatible Bearbeitungs- und Renderregeln. In getrennten PRs optionaler Radius/Einheitensystem, Kilometer/Meter-Abstände. Keine Annahme, dass Aetherys erdgroß ist.

## Verbindliche Arbeitsregeln

1. Lies `docs/aetherys-masterplan.md`, `docs/aetherys-project-guide.md` und relevante bestehende technische Dokumente.
2. Prüfe immer zuerst Code, tatsächliche PR-Integration und Datenbranch; keine Implementierung allein wegen eines veralteten Plandokuments annehmen.
3. **Nur diese Phase bearbeiten**; offene Architekturentscheidungen explizit melden, nicht stillschweigend erraten.
4. Auf eigenem Branch, ein überschaubarer Pull Request; `main` nicht ohne Review verändern.
5. Vorhandene `.vectorcraft`-Weltkarte nicht überschreiben, umbenennen oder ungeprüft migrieren. Falls nötig eine separate Testdatei verwenden.
6. Tests für positive und negative Fälle, Datei-Roundtrip, Undo/Redo und normale Dokumente; `cargo fmt --all --check`, relevante `cargo test` und Clippy soweit durchführbar.
7. Im PR dokumentieren: Dateien/Funktionen geändert, ausgeführte Tests, bekannte Einschränkungen und was **noch nicht** implementiert wurde.

## Abschluss-Checkliste

- [x] Ist-Zustand und Abhängigkeiten geprüft
- [ ] Offene Designfragen entschieden
- [ ] Umsetzung isoliert und rückwärtskompatibel
- [ ] Tests und Format-/Build-Prüfungen dokumentiert
- [ ] PR mit Demonstration oder Beispieldatei
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [ ] Masterplan/Status aktualisiert

## Teil 1 – Radius und Messung

`map.setRadius {"radiusM":1000}` setzt den optionalen Kugelradius in Metern
mit Undo/Redo. `{"radiusM":null}` entfernt ihn. Das vorhandene Dateifeld
bleibt unverändert, ältere Karten brauchen keine Migration.

`map.measure {"from":[179,0],"to":[-179,0]}` nimmt
`[Längengrad, Breitengrad]` in Grad entgegen. Bei 1000 m Radius ergibt
die kürzere Strecke über die Datumsgrenze rund 34,91 m. Die Antwort enthält
Meter, Kilometer und den lokalen horizontalen und vertikalen Maßstab in
Metern pro Dokumentpunkt am Start. Der horizontale Maßstab fällt am Pol
auf null. Ohne Radius ist die Messung nicht verfügbar. Das Modell verwendet
eine Kugel. Kamera, Layer, Zoom-Fades und gespeicherte Geometrien bleiben
von der Messung und vom Setzen des Radius unberührt.

Dieser Teil ändert keine Darstellung oder Bearbeitung von Nahtgeometrien.
Der automatische Umbruch folgt im separaten Naht-PR.
## Bestätigte Regel für den automatischen Umbruch

Der Umbruch ist pro Weltkarte ausdrücklich einschaltbar. Alte Karten behalten
die bisherige Darstellung. Ein Pfad darf in Dokumentpunkten durchgehend über
den linken oder rechten Kartenrand hinauslaufen. Sein Überstand erscheint mit
einem horizontalen Versatz um die Kartenbreite auf der Gegenseite. Linien,
Füllungen und Löcher gehören weiterhin zum selben Vektorobjekt. An den Polen
begrenzt der Kartenrand die Darstellung, ohne Spiegelung oder vertikale
Wiederholung. Die Regel gilt für Ansicht, Bearbeitung und Export.

### Bedienung und Grenzen

In der Modusleiste **Karte an der Datumsgrenze umbrechen** aktivieren oder
`map.setLongitudeWrap {"enabled":true}` aufrufen. Den Pfad über eine
Seitenkante hinaus zeichnen. Ein Klick und ein Ziehen auf der Gegenseite
bearbeiten dieselbe Objekt-ID. Die Modusfilter und Zoom-Fades gelten für beide
Seiten der Kartenansicht. Native Dateien und native Bearbeitungsanhänge
enthalten nur die Originalgeometrie und die Option.

Der Renderer und der SVG/PDF-Ausgabepfad verwenden die beiden angrenzenden
Längenumläufe und schneiden das Ergebnis an der Kartenfläche ab. SVG/PDF
verwenden wie bisher die manuelle Sichtbarkeit. Andere Vektorformate und
separate PSD-Ebenen sind für den Umbruch nicht verifiziert. Auswahlgriffe
bleiben in der ursprünglichen Geometrie. Der Umbruch interpretiert vorhandene
lange Segmente nicht als kurze Verbindungen und erzeugt keine geometrische
Polkappe oder Wiederholungen über mehrere Weltbreiten.

Beispiel auf einer neuen Standardweltkarte: Ein gefülltes Rechteck von
x=990 bis x=1020 zeichnen und umbrechen. Sein rechter Überstand erscheint
von x=0 bis x=20. Bei y<0 wird die Zeichnung an der nördlichen Polkante
beschnitten. Einen geschlossenen Innenring mit Even-Odd-Füllregel hinzufügen,
um ein über die Naht laufendes Loch zu prüfen.

### Gezielte Prüfung

Fünf Renderer-Tests bestanden. Die Nahtprüfung deckt Linie, gefüllte Fläche,
Loch, Polkante, Rasterausgabe, Kartenmodusfilter und Zoom-Fade ab. Der
Engine-Test prüft Auswahl und Ziehen auf der Gegenseite, Undo, nativen
Roundtrip, SVG und PDF. Das PDF enthält weiterhin das unveränderte native
Original als Bearbeitungsanhang. Worldmap-Tests, UI-Buildprüfung und
Formatprüfung bestanden. Die macOS-Nutzerabnahme steht aus.
