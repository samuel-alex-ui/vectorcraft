# AETH-01 – Kartensemantik und geografische Identität

**Stand:** 2026-11-07 · **Status:** Schema vom Nutzer bestätigt; Implementierung und alle zwölf CI-Gates erfolgreich; Performancevergleich dokumentiert; Review und macOS-Nutzerprüfung offen.

Auftrag: [AETH-01](aetherys-tasks/AETH-01.md), Grundlage:
[Masterplan](aetherys-masterplan.md). Bestandsaufnahme:
[PR #10](https://github.com/samuel-alex-ui/vectorcraft/pull/10).

## 1. Verifizierte Basis und Entscheidung

`main` bei `6b3de0a18bb7654e09407ab0cd48b3b10a2b71bd` enthält die grundlegende
Georeferenz. PR #5 (Fade), #6 (Core), #9 (Karte) und #10 (Bestandsaufnahme) sind
offen. AETH-01 benötigt keinen Merge dieser PRs und verändert keine echte Karte.

Der Nutzer hat vor Implementierung ausdrücklich bestätigt: mehrere stabile
Kategorie-IDs pro Objekt/Gruppe, zusätzliche freie Tags, zunächst Textwerte für
eigene Eigenschaften und eine optionale gemeinsame geografische UUID ohne
Geometriesynchronisation. Numerische/boolsche Eigenschaften bleiben eine
spätere versionierte Erweiterung.

`NodeId(u64)` bezeichnet weiterhin einen Knoten innerhalb eines Dokuments.
`Document::alloc_id` und `Document::reid` vergeben für Kopien neue Node-IDs.
Die geografische Entity-ID ist davon unabhängig: zwei unterschiedliche Knoten
können ausdrücklich dieselbe Sache repräsentieren. Gleiche Namen oder ähnliche
Koordinaten erzeugen keine Verknüpfung. Eine Änderung an einer feinen Kontur
verändert niemals automatisch die grobe Kontur.

## 2. Persistentes Schema

- `Document::map_semantics: Option<MapSemantics>` enthält `schema_version: 1`
  und optionale eigene Kategorieanzeigenamen.
- `NodeExtras::geography: Option<Box<GeographicMetadata>>` enthält `categories`,
  `tags`, `properties` und optional `geographic_entity_id`. Der Node-Serializer
  schreibt das optionale Feld `geography` flach am Knoten.
- `Document::world_map` bleibt die ausdrückliche Weltkartenkennung. Ein Name,
  Layerordner oder 2:1-Artboard aktiviert keine Geosemantik.
- Es gibt keine zweite Geometriesammlung und keine abgeleiteten geografischen
  Koordinaten, die mit Pfaden synchronisiert werden müssten.
- Leere/default Metadaten werden weggelassen; ältere und gewöhnliche Dokumente
  bleiben ohne Schema und ohne Geo-Felder. Layer und Gruppen können eigene
  Metadaten tragen; Entity-IDs werden nicht an Kinder vererbt.

Beispiel eines Ausschnitts, **keine** vollständige native Datei:

```json
{
  "map_semantics": {
    "schema_version": 1,
    "categories": {
      "user:6e3a99d5-e361-4bd0-a286-597691803c33": "Handelsrouten"
    }
  },
  "geography": {
    "categories": ["core:river", "core:border"],
    "tags": ["Nordregion"],
    "properties": {"name": "Silberfluss", "navigability": "seasonal"},
    "geographic_entity_id": "7a7eb4cc-b286-44dd-9949-4542254e5ee0"
  }
}
```

`map_semantics` gehört zum Dokument, `geography` jeweils zum Knoten.
Die Beispiel-UUIDs sind frei erfunden und bezeichnen keine vorhandenen Kartenobjekte.

Eingebaute Kategorien: `core:land`, `core:ocean`, `core:lake`, `core:river`,
`core:coastline`, `core:biome`, `core:border`, `core:road`, `core:settlement`,
`core:parcel`, `core:building`. Eigene IDs haben die Form `user:<uuid>`.
Anzeigenamen dürfen sich ändern, IDs bleiben stabil. Definitionen sind Labels,
keine referenzielle Voraussetzung: eine gültige Benutzerkategorie-ID bleibt
auch ohne Anzeigenamen nutzbar. Entfernen eines Labels löscht keine Zuordnungen.

UUID-Text wird in kleingeschriebener Hex-/Bindestrichform angenommen, nil
abgewiesen. Das ist eine Anwendungskonvention auf Basis der
[UUID-Textdarstellung aus RFC 9562](https://www.rfc-editor.org/rfc/rfc9562.html#section-4).
Die Anwendung generiert hier keine UUIDs und legt keine UUID-Version fest.

## 3. Kommandos und Geschichte

| Kommando | Parameter und Verhalten |
|---|---|
| `map.setGeography` | `{ids?|id?, geography: {...}|null}` ersetzt vollständige Metadaten; null/leer entfernt sie. IDs sonst aus der Auswahl. Eine ausdrücklich gemeinsam gesetzte UUID verknüpft unabhängige Darstellungen. |
| `map.setCategory` | `{category: "user:UUID", label: string|null}` definiert/ändert einen Anzeigenamen oder entfernt nur diesen. |
| `map.geographyInfo` | `{ids?|id?}` liest Schema und Zielmetadaten ohne History-/Journaländerung. |

Alle Kommandos sind in der Engine registriert und über CLI/MCP/Control erreichbar.
Noch kein neuer UI-Dialog oder Kartenmodus. Fehlende Targets und ungültige Werte
werden vor dem Commit zurückgewiesen. Änderungen sind atomar und ein Undo-Schritt;
No-ops erzeugen keine neue History. Gewöhnliche Dokumente lehnen die Kommandos ab.
Der aktive Zeichenlayer wird dadurch nicht geändert.

Das erste Setzen nichtleerer Metadaten aktiviert Semantikschema und Inhaltsmarker
atomar. Nach dem Entfernen letzter Metadaten bleibt das Schema erhalten.

## 4. Kopieren und Identität

`Document::reid` löst Entity-Verknüpfungen auf normalen Kopien, statt neue UUIDs
zu erfinden. Interne Entity-Verknüpfungen einer kopierten Auswahl werden ebenfalls
gelöst. Kategorien, Tags und Eigenschaften bleiben in Weltkarten erhalten;
Kategorieanzeigenamen werden nicht automatisch aus dem Quelldokument importiert.
Weltkarten-Ziele erhalten bei Bedarf den Schemamarker. Kopien in gewöhnliche
Dokumente behalten die Vektorgeometrie, jedoch keine rein geografischen Metadaten.
Masken mit Metadaten durchlaufen ebenfalls die Entkopplung. Das Original bleibt
unverändert. Ein gewünschter Zusammenhang wird anschließend ausdrücklich gesetzt.

Die geprüfte Kopieroberfläche ist `edit.duplicate` und der zugrunde liegende
`Document::reid`-Pfad einschließlich Masken. Beliebige Low-Level-`Node::clone`-
Aufrufe sind keine fachliche Kopieroperation. Nicht jeder Generator, jedes
geometrieersetzende Werkzeug, Clipboard-Format oder Plugin besitzt damit bereits
eine vollständige Entity-Policy. Diese Spezialpfade sind vor einem umfassenden
LOD-/Chronica-Vertrag weiter zu prüfen; AETH-07 bleibt eigene Arbeit.

## 5. Reader-Grenzen und Fehlerverhalten

| Sammlung / Text | Grenze |
|---|---:|
| Kategorien pro Knoten | 32 |
| Tags / Eigenschaften pro Knoten | jeweils 64 |
| Eigene Kategorieanzeigenamen pro Dokument | 1024 |
| Kategorie-ID / Eigenschaftsschlüssel | 64 / 128 UTF-8-Bytes |
| Tag / Kategorieanzeigename | 256 UTF-8-Bytes |
| Eigenschaftswert | 4096 UTF-8-Bytes |

Reader begrenzen Einträge während des Lesens; doppelte Listeneinträge/Map-Schlüssel,
falsche Typen und übergroße Werte werden abgelehnt. Tags, Schlüssel und Labels
dürfen nicht leer sein und enthalten keine Steuerzeichen. Eigenschaftswerte sind
Texte und dürfen leer sein. Unbekannte eingebaute Kategorien, ungültige eigene
IDs/UUIDs und nicht unterstützte Schema-Versionen ergeben Fehler. Loader/Writer
validieren die gesamte Semantik, einschließlich gespeicherter Bibliotheks- und
Maskengeometrie. Keine teilweise Anwendung und keine Panics.

Die Grenzen sind technische Anfangsbudgets, keine gemessenen Performancezusagen
für Millionen Objekte. Die vorhandenen Datei-, Knoten- und Ankergrenzen bleiben
zusätzlich wirksam.

## 6. Native Kompatibilität und Versionsschutz

Der untersuchte alte Leser bewahrt unbekannte Dokumentfelder in `Document::extra`,
verwirft jedoch unbekannte Knotenfelder. Ein optionales `geography`-Feld allein
wäre deshalb **nicht** verlustfrei abwärtskompatibel. Ein reines Schemafeld auf
Dokumentebene verhindert den Verlust ebenfalls nicht.

Semantik-Dokumente speichern deshalb zusätzlich `world_map.content_schema: 1`.
Ältere WorldMap-fähige Leser mit dem untersuchten strikten `MapData`-Reader
weisen das unbekannte Feld zurück. Der neue Loader/Writer validiert den Marker
zusammen mit `map_semantics.schema_version`; fehlende/abweichende Marker und
nicht unterstützte Versionen werden zurückgewiesen. Die Envelope-Version wird
nicht erhöht, und der Marker bleibt auch in älteren unterstützten Envelopes
enthalten. Sehr alte Binaries vor Einführung von WorldMap sind nicht geprüft.

Alte Karten werden nicht migriert, umsortiert oder aus Layernamen interpretiert.
Ein verlustbehaftetes Entfernen des Markers ist keine unterstützte Legacy-Save-
Funktion. SVG/PDF bekommen keine neue Garantie zum Erhalt dieser Metadaten.
Ein Top-Level-Registry-Ansatz wurde verworfen: alte Apps könnten dort Daten zwar
erhalten, beim Löschen/Duplizieren aber veraltete Node-ID-Bezüge zurücklassen.

## 7. Prüfung und Grenzen

- Codeanalyse: `NodeId`, `reid`, `Document::world_map`, `NodeExtras`, `NodeWire`,
  Native-Loader/Writer und bestehende Weltkarten-/Property-Tests.
- Zwei Kompatibilitätsproben charakterisieren unbekannte Dokument-/Knotenfelder
  über native Versionen/Kompression und abgelehnte unbekannte Georeferenzfelder.
- Featuretests: verknüpfte unabhängige Geometrie; Metadaten setzen/entfernen,
  Undo/Redo, No-op, Journal-Replay; Kategorien umbenennen; Duplicate entkoppeln;
  normale Dokumente; native Versions-/Kompressions-Roundtrips; ungültige und
  übergroße Eingaben; Property-Tests für fehlerhafte Texte/Versionen.
- Unit-Tests: Maskenkopien und gewöhnliche Ziele, UUID-Validierung sowie
  Inhaltsmarker mit unveränderter Projektion.
- `CARGO_INCREMENTAL=0 RUST_TEST_THREADS=2 cargo xtask ci`: alle zwölf Gates
  erfolgreich (fmt, Workspace-Clippy, 13.065 Unit-/Integrationstests mit 14
  ignorierten Tests, Doctests, No-bundled-fonts, PDF-Abhängigkeiten, Assets,
  Brands, Cursors, i18n-report, Layers, WASM); Rust 1.99.1 auf macOS.
- Der zuletzt erweiterte Format-Testdatensatz mit eigenen Kategorien/Tags wurde
  zusätzlich separat geprüft: fünf Tests erfolgreich. Workspace-Clippy mit
  `--all-targets --keep-going -- -D warnings` danach erneut erfolgreich.
- Ein vorheriger inkrementeller Lauf scheiterte beim Linken des bestehenden
  UI-Tests `zoom_precision` an internen Rust-/LLVM-Symbolen. Der vollständige
  nicht inkrementelle Neubau bestand ohne Anwendungscode-Workaround; der
  betroffene UI-Inkrementalcache wurde bereinigt. Keine CI-Workflow-Änderung.
- Headless CLI-Smoke-Test mit einer separaten synthetischen Karte: Weltkarte
  erzeugen, Rechteck zeichnen, eigene Kategorie/Tags/Textwert/Entity-UUID setzen,
  nativ speichern, in einem zweiten CLI-Prozess wieder öffnen und über
  `map.geographyInfo` vollständig auslesen: erfolgreich. Keine echte Karte benutzt.
- Performance: gepinntes `main` (`6b3de0a18`) gegen dieses PR, jeweils
  `CARGO_INCREMENTAL=0 cargo build --locked -p vectorcraft-cli` im optimierten
  **dev-Profil**, dann `vectorcraft-cli perf` mit 50.000 Pfaden auf demselben Mac.
  Zwei Läufe je Stand, Reihenfolge Basis/PR und PR/Basis mit Pausen. Die unten
  verwendeten vier Läufe hatten keine Lastwarnung (8 Kerne, Start-Load 2,5–4,3).
  Ein erster Lauf mit Lastwarnung wurde verworfen. Binaries wurden getrennt
  aufbewahrt und die PR-Zuordnung zusätzlich durch den erfolgreichen CLI-
  Metadaten-/Datei-Smoke-Test verifiziert.

| 50k-Pfade-Diagnose, zwei Läufe | Basis: Bereich (ms) | PR: Bereich (ms) | Budget (ms) |
|---|---:|---:|---:|
| Nativ speichern | 90,02–94,70 | 91,73–94,32 | 300 |
| Nativ öffnen | 277,29–280,25 | 273,78–280,21 | 300 |
| Render Fit Page | 80,82–128,67 | 114,73–166,40 | 100 |
| Render nach Pan | 85,74–181,53 | 182,49–198,26 | 100 |
| Render 400% | 31,06–41,18 | 33,13–62,13 | 16 |

**Kein allgemeiner Performance-Pass:** die Basis überschritt ein bzw. drei,
das PR in beiden Läufen drei Renderbudgets (CLI-Exit jeweils 1). Alle übrigen
Budgets (Hit-Test, Save/Open, SVG, Pathfinder, Handle-/Segment-Drag) wurden in
allen vier Läufen eingehalten. Save/Open zeigen hier keinen auffälligen
Format-Overhead; die Renderwerte schwanken stark, und Pan bleibt im PR hoch.
Das ist ein dokumentierter Befund, keine Entwarnung für jede Render-Regression
und keine Release-/Großkarten-Performancegarantie. Renderer-Code wurde nicht
geändert; G15 und AETH-09 bleiben offen. Paritätswerte werden nicht angehoben.

Keine echte Karte geändert, keine macOS-Nutzerabnahme behauptet. Keine Kartenmodi,
Sichtbarkeitsfilter, Fade-Vererbung, flexible Detailstufen, automatische Geometrie-
Synchronisation, Globus oder Chronica-Anbindung implementiert. Paritätswerte,
bestehende Illustrator-Gaps und Aufwandsschätzungen ändern sich nicht.

## 8. Bedienbeispiel im Control-Kanal

Auf einer separaten Test-Weltkarte, mit vorhandener Ziel-ID statt `42`:

```json
{"id":1,"method":"engine.execute","params":{"command":"map.setGeography","params":{"id":42,"geography":{"categories":["core:river","core:border"],"tags":["Nordregion"],"properties":{"name":"Silberfluss"},"geographic_entity_id":"7a7eb4cc-b286-44dd-9949-4542254e5ee0"}}}}
{"id":2,"method":"engine.execute","params":{"command":"map.geographyInfo","params":{"id":42}}}
```

Diese Beispielzeilen wurden nicht auf die echte Aetherys-Karte angewendet.

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-07 | Schema vor Implementierung vom Nutzer bestätigt; begrenzte Metadaten, explizite Kommandos, Kopierpolicy und nativer Inhaltsversionsschutz mit Tests. |
