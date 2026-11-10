# AETH-07: LOD-Verknüpfung

**Status:** Implementiert auf eigenem Branch. Review und macOS-Nutzerabnahme offen.

**Reihenfolge:** 07 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Optionale gemeinsame geografische Objekt-ID verknüpft grobe und feine Varianten einer Küste, Stadt usw., während Vektorobjekte getrennt editierbar bleiben. Keine automatische Rückpropagation. Tests für Einordnung, Änderungen, Löschen/Undo und alte Karten.

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
- [x] Offene Designfragen entschieden
- [x] Umsetzung isoliert und rückwärtskompatibel
- [x] Tests und Format-/Build-Prüfungen dokumentiert
- [x] PR mit reproduzierbarem Beispiel
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert

## Bedienung und Grenzen

Die optionale UUID aus AETH-01 bleibt die einzige gemeinsame Identität.
`map.setEntity` setzt oder entfernt sie für explizite Objekt-IDs oder die
Auswahl. Kategorien, Tags, Eigenschaften und Geometrie bleiben erhalten.
`map.entityRepresentations` findet auch ausgeblendete Varianten und liefert
die ID ihrer markierten Detailstufe, außerhalb einer solchen Stufe `null`.

Beispiel mit zwei in einer Testkarte angelegten Objekten (IDs ersetzen):

```json
{"command":"map.setEntity","params":{"ids":[12,24],"entity":"7a7eb4cc-b286-44dd-9949-4542254e5ee0"}}
{"command":"map.entityRepresentations","params":{"entity":"7a7eb4cc-b286-44dd-9949-4542254e5ee0"}}
{"command":"map.setEntity","params":{"id":24,"entity":null}}
```

Die Befehle sind über Engine, Control und MCP verfügbar. Die UUID liefert der
Aufrufer. Es gibt keinen zusätzlichen Dialog und keine automatische Zuordnung
oder Geometriesynchronisierung. Löschen einer Darstellung lässt die übrigen
erhalten. Undo stellt die gelöschte Darstellung mit ihrer UUID wieder her.
Vorhandene Karten und UUIDs benötigen keine Migration. Die Datenbranches aus
PR #7–#9 bleiben unangetastet.


### Prüfung

`cargo test -p vectorcraft-engine --test world_map_semantics`: fünf Tests
bestanden. Der neue Fall deckt zwei Detailstufen, unabhängige Geometrie,
Löschen und Undo, Lösen der Verknüpfung, native Speicherung, alte Weltkarten
ohne Metadaten und die Ablehnung in normalen Dokumenten ab.
`cargo fmt --all --check` bestanden. Keine vollständige CI.
