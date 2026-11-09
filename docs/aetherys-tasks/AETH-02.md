# AETH-02: Detailstufen-Verwaltung

**Status:** Verhalten bestätigt; Implementierung und alle zwölf CI-Gates erfolgreich, Review/Nutzerabnahme offen.
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

- [x] Ist-Zustand und Abhängigkeiten geprüft
- [x] Offene Designfragen entschieden
- [x] Umsetzung isoliert und rückwärtskompatibel
- [x] Tests und Format-/Build-Prüfungen dokumentiert
- [x] PR mit Demonstration oder Beispieldatei
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert

## Bestätigter Entwurf und Nachweis

[Detailstufenvertrag, Kommandobeispiele und Prüfbericht](../aetherys-detail-levels.md). Oberste markierte Layer, fünf Standardgruppen, gezielte Erstellung ohne automatische Migration; Duplizieren löst Entity-Verknüpfungen, Löschen samt Inhalt mit Undo. Nutzer bestätigt am 2026-11-08.

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-08 | AETH-02 beauftragt und Verhalten bestätigt; Implementierung vorbereitet. |
| 2026-11-09 | CI, macOS-Bedienprüfung und Performancevergleich dokumentiert. PR zur Review vorbereitet. |
