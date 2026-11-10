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

## Gruppenindex

Der konservative Bounds-Index aus Draft-PR #6 wird für geeignete
Weltkartengruppen mit 128 bis 262.144 direkten Kindern verwendet. Die
bestehende Cache-Infrastruktur begrenzt die abgeleiteten Indizes auf
64 MiB. Geänderte Gruppen erhalten eine neue Identität. Änderungen an
Symbolen oder Plugins verwerfen die Indizes. Kandidaten werden in der
ursprünglichen Zeichenreihenfolge gerendert.

Zwei gepaarte Läufe auf demselben Build, ohne → mit Index, in ms:

| Fall | 4× | 1.000.000× |
|---|---:|---:|
| 50.000 Pfade | 3,21 → 2,15 / 3,26 → 2,07 | 1,59 → 0,31 / 1,59 → 0,35 |
| 32.000 Stützpunkte | 1,39 → 1,39 / 1,37 → 1,36 | 0,74 → 0,78 / 0,74 → 0,73 |

Messung mit warmem Cache und wechselnder Reihenfolge. Die Systemlast war
hoch. Der Vergleich prüft jeden Frame pixelgenau. Zusätzlich bestanden
der Bildvergleich nach einer Gruppenänderung bei gedrehter Kamera und die
fünf Weltkarten-Renderregressionstests. Aufbaukosten und weitere reale
Lastprofile wurden nicht gemessen. Einzelpfad-Segmentierung,
Tile-Cache und Streaming sind weiterhin offen.

## Nachmessung mit 100.000 Objekten und kaltem Cache

PR #21 ist in `main` (Merge `b91689d01`). Der manuelle Benchmark oben
verwendet ab diesem Folgeschritt 100.000 Rechtecke und einen gefüllten
Küstenpfad mit 256.000 Stützpunkten. Kalt ist der Median aus drei ersten
Frames mit jeweils neuem Renderer. Warm ist der Median aus neun Frames
nach drei Aufwärmframes. Beide Modi laufen abwechselnd und die warmen
Frames werden pixelweise verglichen.

Die vollständige Sortierung an jedem inneren Baumknoten kostete bei
100.000 Rechtecken isoliert 26,22–26,66 ms. Eine Medianpartition senkte
das auf 13,39–13,58 ms (zwei Läufe, neun Messungen je Variante,
optimiertes Profil). Der Baum braucht für die Partition keine vollständige
Sortierung. Die Ausgabe sortiert die Kandidaten weiterhin in Zeichenreihenfolge.

| Fall | Zoom | Warm ohne → mit Index | Kalt ohne → mit Index |
|---|---:|---:|---:|
| 100.000 Rechtecke | 4× | 7,50–7,79 → 4,06–4,26 ms | 47,72–49,30 → 62,34–62,81 ms |
| 100.000 Rechtecke | 1.000.000× | 3,35–3,89 → 0,33–0,40 ms | 41,89–42,41 → 55,92–56,02 ms |
| 256.000 Küstenstützpunkte | 4× | 8,82–8,99 → 8,63–8,97 ms | 16,65–17,82 → 16,14–16,32 ms |
| 256.000 Küstenstützpunkte | 1.000.000× | 3,81–3,84 → 3,81–3,82 ms | 13,20–13,45 → 13,48–13,73 ms |

Die Tabelle fasst zwei Durchläufe mit Medianpartition zusammen. Im
vorherigen Durchlauf kostete der kalte Frame mit Index 81,17 ms bei 4×
und 72,80 ms bei 1.000.000×. Die gepaarten Index-Zuschläge sanken damit
von 30,39 / 28,25 ms auf rund 13–15 ms. Die absoluten Zeiten schwanken
mit der Systemlast. Der Index hielt geschätzt 2.223.848 Byte (2,12 MiB)
für die Rechtecke und 0 Byte für den Einzelpfad. Der maximale RSS des
gesamten Benchmarks mit beiden Dokumenten und Renderpfaden betrug 428 MiB
und enthält auch Fixtures, Geometrie und Pixmaps. Er ist kein Indexwert.

Bildparität nach Gruppenänderung sowie mit Datumsgrenzen-Umbruch,
Zoom-Fade und Modusfilter bestanden. Die fünf Renderregressionstests
für Weltkarten blieben grün. Für den großen Einzelpfad wurde kein
belastbarer Beschleunigungseffekt nachgewiesen. Segment-Culling und
Streaming bleiben offen. Keine Kartendateien geändert.
