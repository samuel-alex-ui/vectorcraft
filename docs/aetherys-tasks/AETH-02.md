# AETH-02: Detailstufen-Verwaltung

**Status:** Geplant – noch nicht an Codex übergeben.  
**Reihenfolge:** 02 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Implementiere auf einer geprüften Basis UI/Engine-Funktionen für beliebige Detailstufen, inkl. Erstellen, Benennen, Löschen und Duplizieren; jede neue Stufe bekommt Standardgruppen `Konturen & Meere`, `Geografie`, `Politik`, `Wirtschaft`, `Infrastruktur`. Keine harte 5er-Grenze, keine stillen Änderungen an Legacy-Maps. Undo/Redo und Speichern/Öffnen testen.

## Verbindliche Arbeitsregeln

1. Lies `docs/aetherys-masterplan.md`, `docs/aetherys-project-guide.md` und relevante bestehende technische Dokumente.
2. Prüfe immer zuerst Code, tatsächliche PR-Integration und Datenbranch; keine Implementierung allein wegen eines veralteten Plandokuments annehmen.
3. **Nur diese Phase bearbeiten**; offene Architekturentscheidungen explizit melden, nicht stillschweigend erraten.
4. Auf eigenem Branch, ein überschaubarer Pull Request; `main` nicht ohne Review verändern.
5. Vorhandene `.vectorcraft`-Weltkarte nicht überschreiben, umbenennen oder ungeprüft migrieren. Falls nötig eine separate Testdatei verwenden.
6. Tests für positive und negative Fälle, Datei-Roundtrip, Undo/Redo und normale Dokumente; `cargo fmt --all --check`, relevante `cargo test` und Clippy soweit durchführbar.
7. Im PR dokumentieren: Dateien/Funktionen geändert, ausgeführte Tests, bekannte Einschränkungen und was **noch nicht** implementiert wurde.

## Abschluss-Checkliste

- [ ] Ist-Zustand und Abhängigkeiten geprüft
- [ ] Offene Designfragen entschieden
- [ ] Umsetzung isoliert und rückwärtskompatibel
- [ ] Tests und Format-/Build-Prüfungen dokumentiert
- [ ] PR mit Demonstration oder Beispieldatei
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [ ] Masterplan/Status aktualisiert
