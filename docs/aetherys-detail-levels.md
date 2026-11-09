# AETH-02: Flexible Detailstufen

**Stand:** 2026-11-08 · **Status:** Implementiert auf eigenem Branch; Prüfbericht unten, Review und macOS-Nutzerabnahme offen.

## Bestätigter Vertrag

Der Nutzer hat vor Implementierung bestätigt: Neue Detailstufen sind ausdrücklich markierte oberste Layer, mit fünf Standardgruppen. Neue und bestehende Karten werden nicht automatisch umgeordnet. Duplizieren kopiert Inhalte und löst geografische Entity-Verknüpfungen; Löschen entfernt Inhalte mit Undo.

AETH-01 / PR #11 ist bei Beginn offen, nicht gemergt. Dieser Branch basiert auf dessen Commit `b8851557127c7577250d0874281f3001e13c81d0`; der PR richtet sich zunächst gegen `codex/aeth-01-geographic-semantics`. `main` bleibt unverändert. Die Karten-Datenbranches #7–#9 und ihre Konflikte aus der [Bestandsaufnahme](https://github.com/samuel-alex-ui/vectorcraft/blob/17ff7d7d75f97004aab57d616b0700a73ba05497/docs/aetherys-inventory.md) werden nicht integriert. Keine reale Kartendatei wird verändert.

## Modell und Bedienung

- `NodeExtras.detail_level: bool` kennzeichnet einen Layer unabhängig vom Namen. Seine `NodeId` bleibt beim Umbenennen erhalten; Kopien erhalten neue Knoten-IDs.
- `map_semantics.detail_levels_version: 1` aktiviert das Formatmerkmal. Der vorhandene WorldMap-Inhaltsmarker bleibt auf Schema 1. AETH-01-Leser lehnen das zusätzliche unbekannte Semantikfeld ab; ältere WorldMap-Leser lehnen den Inhaltsmarker ab. Sehr alte Leser vor WorldMap bleiben unverifiziert.
- Unmarkierte Layer mit Namen wie „Ansicht 1“ werden niemals automatisch übernommen. Fehlende Marker ändern weder gewöhnliche noch alte Karten.
- Neue Stufen heißen standardmäßig „Ansicht 1“, „Ansicht 2“ usw.; vorhandene oberste Layernamen werden für die Standardnamenswahl übersprungen. Eigene Namen dürfen gleich sein, müssen aber nichtleer, ohne Steuerzeichen und höchstens 256 UTF-8-Bytes lang sein.
- Jede neue Stufe enthält normale Gruppen: `Konturen & Meere`, `Geografie`, `Politik`, `Wirtschaft`, `Infrastruktur`. Die Standardgruppen erhalten keine erfundenen Filterkategorien. Ihre Namen und Inhalte sind normal editierbar.
- Im Layerpanel erscheint ausschließlich bei Weltkarten ein aufklappbarer Bereich **Detail Levels / Detailstufen** mit Anlegen, Namensfeld/Umbenennen, Duplizieren und Löschen. Seine Liste scrollt unabhängig vom Layerbaum.
- Anlegen wechselt den Zeichenlayer nicht. Duplizieren verwendet die bestehenden Layer-Kommandofunktionen einschließlich Kopieren von Layerguides und Hervorheben der Kopie. Löschen verwendet bestehende Auswahl-/Guidebereinigung; der letzte Dokumentlayer bleibt geschützt.
- Normale Layeroperationen bleiben verfügbar. Eine ausdrücklich in einen anderen Layer verschobene Stufe behält ihren Marker und bleibt in der Verwaltung sichtbar; die Erstellung selbst ist immer auf oberster Ebene. Es gibt keine neue Zwangssortierung oder Namenserkennung.
- Keine feste Fünfergrenze und kein fachliches Stufenlimit; allgemeine Speicher-, ID- und Dateilesebudgets bleiben bestehen. Das ist keine Zusage unbegrenzter Performance.

## Kommandos und Demonstration

Alle Änderungen laufen durch Engine-Kommandos, sind journalfähig und in einem Schritt rückgängig zu machen. Die Abfrage erzeugt keinen History-/Journaleintrag. Unverändertes Umbenennen erzeugt keinen Historyeintrag.

```json
{"command":"file.newWorldMap","params":{"created":1234567890}}
{"command":"map.detailLevel.create","params":{}}
{"command":"map.detailLevel.list","params":{}}
```

Die zurückgegebene ID für die weiteren Aufrufe einsetzen:

```json
{"command":"map.detailLevel.rename","params":{"id":2,"name":"Stadt"}}
{"command":"map.detailLevel.duplicate","params":{"id":2}}
{"command":"map.detailLevel.delete","params":{"id":2}}
{"command":"edit.undo","params":{}}
```

`id: 2` ist nur ein Beispiel. Für native Demonstrationen eine neue Testkarte speichern, nicht die Aetherys-Karte überschreiben.

## Grenzen

Zoom-Fade und dessen Grenzwerte/Vererbung gehören zu AETH-03. Kartenmodi, Filter, automatische LOD-Synchronisation, Migration vorhandener Ansichtsordner, 3D und Chronica sind nicht Bestandteil dieser Phase. SVG/PDF bekommen keinen neuen Metadaten-Roundtrip-Vertrag. Keine Änderung von CI-Workflows, Paritätsscores oder Aufwandsschätzungen.

## Prüfbericht

- Alle fünf Engine-Integrationstests bestanden, einschließlich Maskenkopie, Schutz des letzten Layers und native Roundtrips.
- Headless-egui: tatsächlicher Klick auf „New Detail Level“ erzeugt eine Stufe; gewöhnliche Dokumente zeigen den Bereich nicht. Test erfolgreich.
- App und CLI im optimierten dev-Profil erfolgreich gebaut (`CARGO_INCREMENTAL=0`).
- CLI in getrennten Prozessen: acht Stufen anlegen, eine umbenennen und duplizieren, speichern, wieder öffnen; neun Stufen mit „Stadt“ und „Stadt copy“ geprüft.
- Native macOS-App mit separatem Control-Port 7987 und synthetischer Testkarte gestartet. Deutsche Oberfläche per Screenshot angesehen: scrollende Verwaltung und Layerbaum sind nutzbar. Anlegen, Namensfeld/Umbenennen, Duplizieren und Löschen per echten egui-Maus-/Tastatureingaben erfolgreich; Cmd+Z stellt die gelöschte Stufe wieder her. Anschließend gespeichert und Testinstanz beendet. Temporäre Screenshots bleiben außerhalb des Repositories.
- Alle zwölf Gates von `cargo xtask ci` bestanden, einschließlich Workspace-Clippy, Tests und WASM. Die macOS-Nutzerabnahme steht aus.
- Performancevergleich mit 50.000 Pfaden auf demselben Mac (acht Kerne, Last 2,5 / 2,4): Speichern 53,38 → 53,79 ms, Öffnen 159,02 → 158,68 ms, jeweils unter 300 ms. Beim Rendern mit 400 % Zoom wurden 15,89 → 16,26 ms gemessen (Budget 16 ms). Alle übrigen Budgets wurden eingehalten. Ein Lauf je Stand erlaubt keine belastbare Aussage über eine Regression.

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-08 | Bestätigten Vertrag, Modell, Kommandos, Bedienung und Grenzen für AETH-02 dokumentiert. |
