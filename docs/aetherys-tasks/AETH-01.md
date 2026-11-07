# AETH-01: Kartensemantik und Objektidentität

**Status:** Schema bestätigt; Metadatenmodell und Kommandos implementiert, alle zwölf CI-Gates erfolgreich; Performancebefunde dokumentiert, Review/Nutzerprüfung offen.
**Reihenfolge:** 01 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Untersuche bestehende `NodeId`, `Document::world_map`, Layerextras/Serialization. Schlage minimales versionierbares Schema für Geo-Kategorien, benutzerdefinierte Eigenschaften und optionale `geographic_entity_id` vor. Priorisiere zunächst Design+Tests und Backward Compatibility. Eine feine Repräsentation darf grobe nie automatisch überschreiben. Entscheidung zu Typen/Tags vor Implementation zurückmelden.

## Verbindliche Arbeitsregeln

1. Lies `docs/aetherys-masterplan.md`, `docs/aetherys-project-guide.md` und relevante bestehende technische Dokumente.
2. Prüfe immer zuerst Code, tatsächliche PR-Integration und Datenbranch; keine Implementierung allein wegen eines veralteten Plandokuments annehmen.
3. **Nur diese Phase bearbeiten**; offene Architekturentscheidungen explizit melden, nicht stillschweigend erraten.
4. Auf eigenem Branch, ein überschaubarer Pull Request; `main` nicht ohne Review verändern.
5. Vorhandene `.vectorcraft`-Weltkarte nicht überschreiben, umbenennen oder ungeprüft migrieren. Falls nötig eine separate Testdatei verwenden.
6. Tests für positive und negative Fälle, Datei-Roundtrip, Undo/Redo und normale Dokumente; `cargo fmt --all --check`, relevante `cargo test` und Clippy soweit durchführbar.
7. Im PR dokumentieren: Dateien/Funktionen geändert, ausgeführte Tests, bekannte Einschränkungen und was **noch nicht** implementiert wurde.

## Designstand (2026-11-07)

[Schemaentwurf und Testplan](../aetherys-semantics-design.md): bestehende IDs/Serialisierung geprüft, Kategorien/Tags/Textwerte und optionale Entity-UUID vorgeschlagen. Unbekannte Knotenfelder sind bei älteren Lesern verlustgefährdet; Versionsschutz gesondert beschrieben. Der Nutzer hat das Schema bestätigt. Implementiert sind Metadatentypen, drei Engine-Kommandos und Inhaltsversionsschutz; siehe den aktuellen Vertrag in Abschnitten 2–6 des Berichts.

## Abschluss-Checkliste

- [x] Ist-Zustand und Abhängigkeiten geprüft
- [x] Schemaentscheidung vor Implementierung bestätigt (mehrere Kategorien, Tags, Textwerte, optionale Entity-UUID)
- [x] Umsetzung ohne Kartenmigration; alte/gewöhnliche Dateien behalten leere Metadaten
- [x] Tests und Format-/Build-Prüfungen dokumentiert (alle zwölf CI-Gates erfolgreich, CARGO_INCREMENTAL=0)
- [x] PR mit Control-Beispiel, synthetischen Tests und erfolgreichem CLI-Save/Open-Smoke-Test vorbereitet
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-07 | Schemaentwurf und Kompatibilitätsproben vorbereitet; Nutzer bestätigt Schema; Implementierung und Tests ergänzt, Review/Nutzerprüfung offen. |
