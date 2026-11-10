# AETH-10: 3D-Globusvorschau

**Status:** In Vorbereitung. Technischer Plan zur Abstimmung; Implementierung ausstehend.

**Reihenfolge:** 10 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Nach tragfähiger Georeferenz und Nahtregeln: separate drehbare/zoombare read-only 3D-Vorschau. Gleiche kanonische 2D-Geometrie, keine zweite Geometriequelle, keine 3D-Zeichenwerkzeuge, kein Terrainrelief erforderlich.

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

## Implementierungsplan

1. **Kartentextur aus dem 2D-Renderer.** Die geografische Referenzfläche
   `WorldMap::size()` über `Renderer::render_region_with` in eine begrenzte
   equirektanguläre Vorschautextur zeichnen (anfangs 1024 × 512 Pixel).
   Die Vorschau übernimmt den aktiven Kartenmodus, die manuelle Sichtbarkeit
   und die Detailstufen am aktuellen 2D-Zoom über `world_map_zoom`.
   Der Bearbeitungsschalter für ausgeblendete Inhalte bleibt rein im Editor.
   Kartenoption und Renderer bestimmen den Datumsgrenzen-Umbruch.
2. **Kugel in einem reinen Rust-Modul abbilden.** Pro Bildpunkt der
   orthografischen Scheibe die vordere Kugeloberfläche berechnen, mit der
   Kamerarotation in Länge/Breite umrechnen und über die vorhandene
   Kartenprojektion die Textur abtasten. Horizontal periodisch abtasten,
   vertikal an den Polen begrenzen. Die portable CPU-Abbildung braucht keine
   neue Grafik-API. `vectorcraft-three-d` erzeugt heute Vektorfacetten für
   Dokumenteffekte und bleibt davon getrennt.
3. **Separates Vorschaufenster.** Ein Weltkartenbefehl öffnet die Vorschau.
   Ziehen dreht den Globus, Scrollen zoomt die Vorschau. Eine explizite
   Aktion zentriert die 2D-Kamera auf den betrachteten Punkt. Die
   Globuskamera wird pro Vorschau gehalten. Nur Dokumentwechsel, Änderungen
   der Kartendarstellung oder ein geänderter 2D-Detailzoom machen die
   Kartentextur ungültig. Rotation und Globuszoom verwenden die vorhandene
   Textur. Es entstehen weder Dokumentänderungen noch Undo-Schritte.
4. **Gezielt prüfen.** Synthetische Karte mit Markierungen an Äquator,
   Datumsgrenze und Polen verwenden. Orientierung, Naht, Rotation, Zoom
   und Übergabe an die 2D-Kamera prüfen. Dazu Tests für Modus/Fade,
   Cache-Ungültigkeit und gewöhnliche Dokumente. Dokumentinhalt und
   Undo-Historie vor und nach der Navigation vergleichen.

Die Vorschau verwendet eine abgeleitete Rastertextur. Bei starker
Vergrößerung wird sie unscharf. Der Globuszoom lädt zunächst keine feinere
Detailstufe. Die 2D-Kamera bestimmt die gezeigte Detailstufe. Ein
Planetenradius ist für die Darstellung der Einheitskugel optional.
Über die Naht falsch gezeichnete Altgeometrie wird nicht automatisch
umgedeutet. Terrainrelief, 3D-Bearbeitung und Streaming gehören nicht
zum ersten Umfang.
