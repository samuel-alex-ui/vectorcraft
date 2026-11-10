# AETH-05: Modus-Sichtbarkeitsfilter

**Status:** Implementierung auf eigenem Branch. Review und macOS-Nutzerabnahme stehen aus.
**Reihenfolge:** 05 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Pro Kartenmodus können Nutzer Gruppen und semantische Eigenschaften aktivieren/deaktivieren. Filter gelten für die ganze Welt und alle Detailstufen. Effektive Anzeige = manuelle Sichtbarkeit × Modusregel × Zoom-Fade. Filter auf stabile Kategorien/IDs, nicht bloß Layername. Kein destruktives Umschreiben von Vektoren/Farben.

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
- [x] PR mit Demonstration oder Beispieldatei
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert

[Bedienung, Regeln und Demonstration](../aetherys-mode-filters.md).

| Datum | Änderung |
|---|---|
| 2026-11-10 | Implementierung, macOS-Test und Pflicht-CI dokumentiert. |
