# AETH-09: Performance für Großkarten

**Status:** Begonnen. Die Render-Baseline für den aktuellen Hauptbranch liegt zur Review vor.

**Reihenfolge:** 09 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Prüfe zuerst Draft PR #6. Integriere nur validierte, testbare Teile für Sichtbereich, Indizes, Tiles/Cache und eventuell Segment-Culling. Realistische Benchmarks an 1.000.000×, vielen Objekten und großen Einzelpfaden. Ungeprüftes Streaming nicht als fertig darstellen.

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

## Erster Schritt: Render-Baseline

Draft-PR #6 enthält einen begrenzten Gruppenindex, Tile-Adressierung und einen
separaten Vorschau-Cache. Die früher gemeldete Indexmessung entstand auf
einem älteren Stand. Der Draft enthält außerdem eine Kartendatei und bereits
durch spätere Phasen abgedeckte Funktionen. Für diesen Schritt wurde nur
das Benchmark-Konzept übernommen.

```sh
cargo test -p vectorcraft-render --test world_map_perf -- --ignored --nocapture
```

Synthetische Weltkarte mit 50.000 Rechtecken oder einem gefüllten Küstenpfad
mit 32.000 Stützpunkten. Pro Fall drei Aufwärmframes, neun Messframes und
24 Pixel horizontaler Versatz zwischen drei Kamerapositionen. Bildgröße
1280 × 720. Der Küsten-Ausschnitt liegt bei 1.000.000× direkt auf der Kante.

| Fall | 4× | 1.000.000× |
|---|---:|---:|
| 50.000 Pfade | 2,95 / 2,96 ms | 1,87 / 2,01 ms |
| 32.000 Stützpunkte | 1,47 / 1,50 ms | 0,78 / 0,87 ms |

Die Werte sind Mediane zweier lokaler Läufe im optimierten Testprofil bei
hoher Systemlast. Sie dienen als Diagnose. Der Test prüft sichtbaren
Bildinhalt und setzt keine Zeitgrenze. Filter und Nahtumbruch sind aus.
Ein Leistungsgewinn durch einen Index, Segment-Culling oder Streaming ist
damit noch nicht nachgewiesen.
