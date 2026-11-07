# AETH-00: Bestandsaufnahme

**Status:** Bestandsaufnahme erstellt – Review und macOS-Nutzerprüfung ausstehend.
**Reihenfolge:** 00 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Prüfe `main`, offene PR #5–#9, Branch-Abhängigkeiten und die letzte maßgebliche `.vectorcraft`-Datei. Liefere Bericht mit Tabelle: Feature, Branch, Status, Tests, Konflikte, Datenquelle. Nichts mergen, nichts an Karten ändern. Bestimme empfohlene gemeinsame Entwicklungsbasis.

## Verbindliche Arbeitsregeln

1. Lies `docs/aetherys-masterplan.md`, `docs/aetherys-project-guide.md` und relevante bestehende technische Dokumente.
2. Prüfe immer zuerst Code, tatsächliche PR-Integration und Datenbranch; keine Implementierung allein wegen eines veralteten Plandokuments annehmen.
3. **Nur diese Phase bearbeiten**; offene Architekturentscheidungen explizit melden, nicht stillschweigend erraten.
4. Auf eigenem Branch, ein überschaubarer Pull Request; `main` nicht ohne Review verändern.
5. Vorhandene `.vectorcraft`-Weltkarte nicht überschreiben, umbenennen oder ungeprüft migrieren. Falls nötig eine separate Testdatei verwenden.
6. Tests für positive und negative Fälle, Datei-Roundtrip, Undo/Redo und normale Dokumente; `cargo fmt --all --check`, relevante `cargo test` und Clippy soweit durchführbar.
7. Im PR dokumentieren: Dateien/Funktionen geändert, ausgeführte Tests, bekannte Einschränkungen und was **noch nicht** implementiert wurde.

## Ergebnis (2026-11-07)

[Bestandsaufnahme mit Featuretabelle, Branch-Abhängigkeiten, Kartenprüfsummen und Basisempfehlung](../aetherys-inventory.md). Keine Merges, Kartenänderungen oder neuen Produktentscheidungen.

## Abschluss-Checkliste

- [x] Ist-Zustand und Abhängigkeiten geprüft
- [ ] Offene Designfragen entschieden
- [x] Umsetzung isoliert und rückwärtskompatibel (nur Dokumentation)
- [x] Tests und Format-/Build-Prüfungen dokumentiert (alle 12 CI-Gates erfolgreich)
- [x] PR vorbereitet; Demonstration als dokumentierte Git-/Kartenbefunde, keine neue Kartendatei
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-07 | Bestandsaufnahme und Basisempfehlung erstellt; Review, Produktentscheidungen und Nutzerprüfung bleiben offen. |
