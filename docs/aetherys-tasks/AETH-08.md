# AETH-08: Naht, Pole und Maße

**Status:** Teil 1 (Radius und Messung) umgesetzt. Naht- und Polgeometrie in Abstimmung.

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
Die explizite Naht- und Polregel wird getrennt abgestimmt.
## Bestätigte Regel für den automatischen Umbruch

Der Umbruch ist pro Weltkarte ausdrücklich einschaltbar. Alte Karten behalten
die bisherige Darstellung. Ein Pfad darf in Dokumentpunkten durchgehend über
den linken oder rechten Kartenrand hinauslaufen. Sein Überstand erscheint mit
einem horizontalen Versatz um die Kartenbreite auf der Gegenseite. Linien,
Füllungen und Löcher gehören weiterhin zum selben Vektorobjekt. An den Polen
begrenzt der Kartenrand die Darstellung, ohne Spiegelung oder vertikale
Wiederholung. Die Regel gilt für Ansicht, Bearbeitung und Export.

Für die Umsetzung müssen Renderkopie und Auswahl denselben Versatz verwenden.
Eine Bearbeitung auf der Gegenseite muss die Mausposition ins Original
zurückrechnen und den Versatz über den gesamten Drag beibehalten. Raster- und
Vektorexport müssen dieselbe Kartenkante verwenden. Der native Dateiexport
speichert Originalgeometrie und die Kartenoption, keine Kopien.
