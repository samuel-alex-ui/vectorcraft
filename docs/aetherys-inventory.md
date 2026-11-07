# AETH-00 – Bestandsaufnahme und empfohlene Entwicklungsbasis

**Stand:** 2026-11-07 · **Status:** Bestandsaufnahme zur Prüfung; keine Integrationsfreigabe.

Auftrag: [AETH-00](aetherys-tasks/AETH-00.md), Grundlage: [Masterplan](aetherys-masterplan.md).
Untersucht wurden GitHub-PR-Metadaten, frisch gefetchte Git-Objekte, Quellcode,
Tests und native JSON-Kartendateien. Keine Branches zusammengeführt, keine Karte
verändert, keine Nutzerprüfung oder visuelle Abnahme behauptet. Die lokalen
`plan/STATUS.md`, `plan/execution-plan.md` und `plan/architecture.md` fehlen.

## 1. Fixierter Ausgangsstand

| Ref | Commit beim Audit |
|---|---|
| `main` | `6b3de0a18bb7654e09407ab0cd48b3b10a2b71bd` |
| `feature/aetherys-zoom-fade` (#5) | `2ddf0d39cd57641e3cdc029726dfe5191fbb2aa9` |
| `feature/aetherys-worldmap-core` (#6) | `db0358a493dfa55d8002a2c11f9ed4d51f3df401` |
| `chatgpt/aetherys-coastline-halo` (#7) | `0c872dd865ceded2f4b58a89cbf2e3c2366386b8` |
| `chatgpt/aetherys-illustrated-atlas` (#8) | `4fc68357b59bc14ee0ad300ad0a7abb8a0670fd2` |
| `chatgpt/aetherys-detailed-vector-terrain` (#9) | `ea0f4499cc98d07efe70c3057ff9d41ba74d829a` |

`main` enthält die Weltkartenbasis einschließlich Zoom aus #3/#4, aber weder
Zoom-Fade noch Worldmap-Core noch eine Datei unter `maps/`. Seit dem gemeinsamen
Vorfahren `a018f564c37125d26d052c1c9b48e37829306bf2` kamen auf `main` die
Masterplan-/Auftragsdokumente hinzu; #5 hat einen eigenen Implementierungszweig.

## 2. Feature, Branch, Status, Tests, Konflikte, Datenquelle

| Feature | Branch / PR | Tatsächlicher Status | Tests / Evidenz | Konflikte / Abhängigkeit | Datenquelle |
|---|---|---|---|---|---|
| Georeferenz, Weltkartenerzeugung, Präzision, Zoom bis 1.000.000× | `main`, frühere #1–#4 | Code vorhanden; keine umfassende Hauszoom-/Lastgarantie | `crates/worldmap/tests/precision.rs`, `crates/format/tests/world_map.rs`, Engine-Journal- und UI-Menütests; aktueller Prüflauf unten | Grundlage aller Zweige | `Document::world_map`, `crates/worldmap`, `crates/ui-egui/src/state.rs` |
| Unabhängiges Zoom-Fade auf Layern | `feature/aetherys-zoom-fade`, [#5](https://github.com/samuel-alex-ui/vectorcraft/pull/5) | Offen, **kein** Draft laut API (Titel/PR-Text sagt noch draft), nicht in `main` | Unit-, Engine-/Format-/Render-`world_map_fade` und Layerdialogtests vorhanden; in AETH-00 nicht auf diesem Head ausgeführt | Baut auf altem `main` auf; lokaler Merge-Test mit aktuellem `main` sauber | `NodeExtras::zoom_fade`, Hit-Test, Renderoption, Layerbefehle; [Details](https://github.com/samuel-alex-ui/vectorcraft/blob/db0358a493dfa55d8002a2c11f9ed4d51f3df401/docs/aetherys-zoom-fade.md) |
| Räumlicher Index, Tile-/Cache-Bausteine, Maßstab, offene Linien an Weltnaht | `feature/aetherys-worldmap-core`, [#6](https://github.com/samuel-alex-ui/vectorcraft/pull/6) | Offener Draft; Code vorhanden, kein vollständig gestreamter Welteneditor | `world_map_spatial`, `spatial_budget`, Worldmap-Tests zu Tiles, Cache, Manifest, Store, Maßstab, Naht vorhanden; nicht frisch auf diesem Head ausgeführt; Remote-CI siehe unten | #5 ist Vorfahr; #6 enthält inzwischen auch eigene Kartenänderungen | `crates/render/src/spatial.rs`, `crates/worldmap`; [Details](https://github.com/samuel-alex-ui/vectorcraft/blob/db0358a493dfa55d8002a2c11f9ed4d51f3df401/docs/aetherys-worldmap-core.md) und [Cache](https://github.com/samuel-alex-ui/vectorcraft/blob/db0358a493dfa55d8002a2c11f9ed4d51f3df401/docs/aetherys-tile-cache-store.md) |
| Cyan-Küstensaum | `chatgpt/aetherys-coastline-halo`, [#7](https://github.com/samuel-alex-ui/vectorcraft/pull/7) | Offen; Kartenänderung, kein separates App-Feature | JSON-Struktur/IDs/Prüfsumme geprüft; keine native oder visuelle Abnahme | Ziel #6 ist weitergelaufen: **echter Kartenkonflikt** | Native Karte, Blob-Tabelle unten |
| Atlas/Biome, inzwischen fünf Ansichtsordner | `chatgpt/aetherys-illustrated-atlas`, [#8](https://github.com/samuel-alex-ui/vectorcraft/pull/8) | Offen; neuer als PR-Text „11 Ebenen“: jetzt 5 Hauptlayer, 36 Layer insgesamt | JSON-Struktur/IDs/Prüfsumme geprüft; keine native oder visuelle Abnahme | #7 ist Vorfahr; lokaler Merge-Test sauber | Native Karte, Blob-Tabelle unten |
| Detailliertes Terrain, fünf Ansichtsordner | `chatgpt/aetherys-detailed-vector-terrain`, [#9](https://github.com/samuel-alex-ui/vectorcraft/pull/9) | Offener Draft; neueste untersuchte Datenvariante, nicht automatisch autoritativ | JSON-Struktur/IDs/Prüfsumme geprüft; keine native oder visuelle Abnahme | Ziel #8 ist weitergelaufen: **echter Kartenkonflikt**; zusätzliche Python-Skripte/eigene CI-Jobs | Native Karte, Blob-Tabelle unten |
| Flexible Detailstufen, semantische Kategorien, Kartenmodi, LOD-Identität, Globus | AETH-01–11 | Kein Nachweis einer fertigen Umsetzung auf `main` oder durch die Datenordner | Separate Aufträge erforderlich | Produktentscheidungen im Masterplan offen | [Auftragsübersicht](aetherys-tasks/README.md) |

Die Datenzweige #7 und #8 unterscheiden sich vom jeweiligen aktuellen Vorgänger
nur in der Karte. #9 ergänzt gegenüber #8 außerdem
`scripts/enrich_aetherys_atlas.py`, `scripts/reorganize_aetherys_views.py` und zwei
`.github/workflows/*aetherys*.yml`. Diese sind vorhandener Altbestand, werden hier
weder ausgeführt noch übernommen. Neue Integration muss die Rust-only- und
Shared-Workflow-Regeln berücksichtigen; keine pauschale Übernahme von #9 als App-Basis.

### Remote-CI der aktuellen Feature-Heads

- #5: Die Abfrage nach PR-ausgelösten Runs lieferte keine Runs; das ist kein
  Beweis, dass nie getestet wurde.
- #6: [CI-Run 42260360097](https://github.com/samuel-alex-ui/vectorcraft/actions/runs/42260360097)
  am fixierten Head: `fmt`, `clippy`, `test`, `doctests`, `no-bundled-fonts` und
  `pdf-deps` erfolgreich. **Gesamtstatus failure:** `assets` meldet fehlenden
  `ASSETS.md`-Eintrag für `maps/Aetherys World Map2.vectorcraft`; nachfolgende
  Gates `brands`, `cursors`, `i18n-report`, `layers`, `wasm` wurden übersprungen.
  Andere Runs (Nix, SonarCloud, Desktop printing, Affinity corpus) erfolgreich.
  Dies ist ein Remote-Nachweis vom 2026-11-06, kein neuer lokaler Feature-Test.
- #9: abgefragte Runs CI, Packaging lint und SonarCloud enden mit
  `action_required`, nicht mit einem erfolgreichen Qualitätsnachweis.
- Die API liefert nur die erste Seite und hier PR-ausgelöste Runs. Vor einer
  Datenintegration müssen Urheberschaft/Quelle/Lizenz geklärt und die Karte gemäß
  Asset-Regeln zugeordnet werden; keine Provenienz aus Dateinamen ableiten.

## 3. Abhängigkeiten und tatsächliche Konflikte

PR-Ziele: `main ← #5 ← #6 ← #7 ← #8 ← #9`. Das ist eine **PR-Zielkette**, keine
vollständig lineare aktuelle Commit-Kette. `git merge-base --is-ancestor` bestätigt
#5→#6 und #7→#8, verneint aber aktuelles #6→#7 und aktuelles #8→#9.

- #6 hat gegenüber dem in #7 enthaltenen Stand die Kartencommits `b60c4e4cc`
  und `db0358a49` ergänzt.
- #8 hat gegenüber dem in #9 enthaltenen Stand `21fb8d736` und `4fc68357b`
  ergänzt. Daher ist „#9 enthält schon alles aus #8“ falsch.
- `git merge-tree --write-tree --name-only BASE HEAD` (ohne Checkout/Merge)
  ergibt Exit 0 für main/#5, #5/#6, #7/#8; Exit 1 für #6/#7 und #8/#9,
  jeweils ausschließlich `maps/Aetherys World Map2.vectorcraft`.
- Die GitHub-Anbindung meldete `mergeable=false` für #5/#7/#8/#9 und `true`
  für #6. Bei #5/#8 widerspricht das dem lokalen Merge-Test der gefetchten
  aktuellen Heads. API-Mergebarkeit ist daher hier kein verlässlicher Beleg
  eines Textkonflikts; vor Integration erneut prüfen.

Ein späterer Rebase der Codeänderungen ist getrennt vom Abgleich der
Kartenversionen zu planen. Kein `ours`/`theirs` auf die ganze Karte und kein
automatisches Ausführen der Umstrukturierungsskripte.

## 4. Kartendatei: Herkunft und Integrität

Pfad auf allen vier Datenzweigen: `maps/Aetherys World Map2.vectorcraft`.
Alle sind native `vectorcraft`-JSON-Dokumente mit equirektangulärer Referenz
1000 × 500 und Radius 6.371.000 m. Diese gespeicherte Radiuswahl ist eine
Eigenschaft dieser Dateien, keine globale Produktvorgabe.

| PR | Git-Blob | Bytes | Hauptlayer / alle Layer | Pfade / Anker |
|---|---|---:|---:|---:|
| #6 | `d14208893b7aea368f530bfa63ec2461b04fb81d` | 2.252.447 | 11 / 11 | 465 / 27.342 |
| #7 | `5e8d80d9302c4dfa3caf568a831b1c73cddf0fc0` | 832.315 | 3 / 3 | 419 / 11.798 |
| #8 | `bd1234c067cc8627d07e28e1a4391c84a22dd034` | 2.078.754 | 5 / 36 | 468 / 25.417 |
| #9 | `c8cb8943d89c7f6314cb1764b4c57b5c52247a37` | 6.856.052 | 5 / 39 | 484 / 167.373 |

SHA-256 der unveränderten Dateibytes:

```text
#6 c432c51bb063dcc224bd941df1a6d56d4fa39a362d838f52e974e0ad844b1e2f
#7 a9e2e716970e49c4873c97ae734fb941d8597f10235b54111c18c15aca233dc8
#8 b8cfc1d4267705950ee19a19f850eec62ca877bd6d1ebf07a6ab5c0835b3fadd
#9 faa58a6ff2351b602b171cce015b05251e080c7b867a6b3270eb6810f2f141b6
```

Rekursiv über `kind.children` geprüft: eindeutige Node-IDs und `next_id` größer
als die maximale vorhandene ID auf allen vier Ständen. **Keiner** speichert
`zoom_fade`. Die Ansichtsordner auf #8/#9 haben jeweils die Unterlayer „Kontur
und Meere“, „Geografie“, „Politik“, „Wirtschaft“; Ansicht 2–5 sind auf beiden
Heads leer. Das beweist weder fünf funktionierende LODs noch ererbte Fade-Regeln,
strukturierte Kategorien oder Kartenmodi. Zwei Textobjekte sind auf #6/#8/#9
vorhanden; sämtliche Zahlen sind Strukturmessungen, keine inhaltliche Abnahme.

**Empfohlener Datenkandidat:** #9 am oben fixierten Commit als neueste detaillierte
Arbeitsvariante separat erhalten. #8 ebenfalls unverändert als abweichenden
Vergleichs-/Sicherungsstand erhalten. Welche Datei maßgeblich wird, muss nach
inhaltlichem Vergleich und Nutzerprüfung entschieden werden. Der Befund erlaubt
keine verlustfreie automatische Zusammenführung und keine stille Migration.

## 5. Empfohlene gemeinsame Entwicklungsbasis und nächste Entscheidungen

1. **App-Code:** aktuelles `main` (`6b3de0a18…`) als gemeinsame Referenz, mit allen
   Masterplan-/Auftragsdokumenten. Dieser reine Dokumentations-PR basiert darauf.
2. **Integration:** #5 zuerst auf aktualisierter Basis prüfen/rebasen, dann die
   Codebestandteile von #6 einschließlich gezielter Tests. Dessen Kartendatei
   ausdrücklich separat behandeln. Erst nach Review eine gemeinsame integrierte
   Codebasis festlegen; AETH-00 nimmt keinen Merge vor.
3. **Kartendaten:** #9 und #8 anhand der fixierten Blobs erhalten; zunächst nur
   separate temporäre Testkopien für native Öffnen-/Speichern-/Renderprüfungen.
   Keine echte Karte in die neue Entwicklungsbasis kopieren oder überschreiben.
4. **AETH-01:** vor Implementierung minimales Schema für Kategorien, eigene Tags
   und optionale geografische Identität vorlegen. Vorhandene Node-IDs sind nicht
   bereits die gemeinsame Identität verschiedener Detailrepräsentationen.
5. Offen bleiben verbindliche Wahl der Kartenversion, Kategorien/Tags,
   Editierzugriff auf verborgene Objekte, LOD-Compositing, Naht/Pol-Semantik und
   Migrations-/Exportregeln. Diese Bestandsaufnahme trifft keine neuen
   Produktentscheidungen. Modusstile, Globus und Chronica bleiben spätere Arbeit.

## 6. Prüfung und Grenzen

- GitHub-PRs #5–#9 und Remote-Refs abgefragt; Git-Abstammung, Baumdifferenzen
  und virtuelle Merge-Tests wie oben ausgewertet. Keine Ref durch diese Tests verändert.
- JSON aller vier Karten aus `git show REF:PATH` gelesen, SHA-256 über die
  Rohbytes berechnet, Layer/Pfade/Anker rekursiv gezählt und IDs geprüft.
  JSON-Lesbarkeit ersetzt **keinen** nativen Loader-/Roundtrip-Test.
- `CARGO_TARGET_DIR=/Users/samuelalex/Developer/GitHub/vectorcraft/target
  RUST_TEST_THREADS=2 cargo xtask ci` auf der unveränderten Codebasis von `main`:
  **alle 12 Gates erfolgreich** (`fmt`, `clippy`, `test`, `doctests`,
  `no-bundled-fonts`, `pdf-deps`, `assets`, `brands`, `cursors`, `i18n-report`,
  `layers`, `wasm`). 13.051 Unit-/Integrationstests erfolgreich, 14 ignoriert,
  0 Fehler im Workspace-Testgate; zusätzlich die No-bundled-fonts-Tests erfolgreich.
  Rust 1.99.1 auf macOS. Das prüft ausdrücklich nicht die offenen Feature-Heads.
- Kein neuer Produktcode, kein neuer Parser, keine Fixture und keine Assets.
  Feature-spezifische neue Positiv-/Negativ-, Undo-/Redo- und Roundtrip-Tests
  entfallen für diese reine Dokumentationsphase. Bestehende native Tests auf
  `main` decken Geometadaten, beschädigte Metadaten und gewöhnliche Dokumente ab.
- Keine App gestartet, keine Kartendarstellung abgenommen, keine macOS-Nutzerprüfung.
  Die Abnahme-Checkboxen bleiben bis zum Review offen. Paritätswerte und
  Meilensteine ändern sich durch die Bestandsaufnahme nicht.

Reproduzierbare Git-Prüfungen (für jeden Tabellen-Ref entsprechend wiederholen):

```sh
git fetch origin
git rev-parse origin/main origin/feature/aetherys-worldmap-core
git merge-base origin/main origin/feature/aetherys-zoom-fade
git merge-base --is-ancestor origin/feature/aetherys-worldmap-core origin/chatgpt/aetherys-coastline-halo
git merge-tree --write-tree --name-only origin/feature/aetherys-worldmap-core origin/chatgpt/aetherys-coastline-halo
git ls-tree -rl origin/chatgpt/aetherys-detailed-vector-terrain maps/
git show 'origin/chatgpt/aetherys-detailed-vector-terrain:maps/Aetherys World Map2.vectorcraft' | shasum -a 256
RUST_TEST_THREADS=2 cargo xtask ci
```

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-07 | AETH-00: aktuelle Code-/Datenstände, PR-Abhängigkeiten, Kartenprüfsummen und Integrationsvorschlag dokumentiert; keine Karten- oder App-Änderung. |
