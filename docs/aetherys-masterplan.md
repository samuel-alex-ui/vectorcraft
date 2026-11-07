# Aetherys / VectorCraft – Masterplan und Codex-Roadmap

**Stand:** 2026-11-07  
**Status:** Produktentscheidungen und geplante Arbeit; KEINE Bestätigung, dass die neuen Funktionen bereits implementiert sind.  
**Basis:** [Bisheriger Projektleitfaden](aetherys-project-guide.md) · [Technischer Weltkartenstatus](aetherys-world-map.md)

> **Ziel:** Eine einzige, geografisch konsistente Fantasywelt vom ganzen Planeten über Kontinente und Städte bis zu Grundstücken und Häusern in einem nativen 2D-Vektoreditor bearbeiten. Ein separater, nur lesender 3D-Globus dient später der Vorschau. Bestehende VectorCraft-Werkzeuge und Daten bleiben nutzbar.

## A. Verbindlich beschlossene Produktziele

1. **Eine Welt, eine geografische Referenz.** Equirektanguläre 2:1-Weltkarte als Start, geografische Längen/Breiten, optionaler Planetenradius; keine unverbundenen Einzelkarten.
2. **2D-Zeichnung, 3D-Vorschau.** Keine direkte 3D-Bearbeitung, kein Terrainrelief als Voraussetzung.
3. **Extreme Detailtiefe.** Vom Planeten bis zum Haus. Zielzoom 1.000.000×; Zoomfähigkeit ist nicht gleichbedeutend mit bereits verifizierter Performance bei Millionen Objekten.
4. **Beliebig viele Detailstufen.** Anfangs „Ansicht 1–5“, später Ansicht 6, 7 usw. jederzeit einfach hinzufügen, umbenennen, entfernen und Zoomgrenzen anpassen können. Keine harte Beschränkung auf fünf.
5. **Wiederkehrende Layergruppen je Detailstufe.** Standardvorlage: „Konturen & Meere“, „Geografie“, „Politik“, „Wirtschaft“, optional „Infrastruktur“. Beim Erzeugen einer neuen Detailstufe werden diese automatisch angelegt. Bestehende Karten nicht ohne explizite Migration umsortieren.
6. **Zoom-Vererbung.** Jede übergeordnete Detailstufe hat einen Bereich für Zoom-Fade (Einblend-/Ausblend-Start und -Ende); Untergruppen/Objekte erben diesen zunächst. Individuelle Ausnahmen sind möglich. **Keinen separaten, redundanten Weichheits-Slider bauen:** Die vorhandenen Fade-Grenzen definieren den Übergang bereits.
7. **Kontur-Verfeinerung.** Grobe Kontur bei Weltzoom und feinere Kontur in höheren Detailstufen können dieselbe geografische Sache repräsentieren. Feine Bearbeitung schreibt Änderungen NICHT automatisch in die grobe Kontur zurück. Für größere Änderungen zuerst grobe Stufe bearbeiten, anschließend feinere. Optionale stabile Verknüpfung zwischen den Repräsentationen ist zu spezifizieren.
8. **Kartenmodi sind UNABHÄNGIG von Detailstufen und Layerbaum.** Standardvorlagen: Geografisch, Biome, Politisch, Klima, Wirtschaft, Infrastruktur; eigene Modi erstellen, umbenennen, duplizieren und konfigurieren. Modi bleiben beim Zoomen aktiv. Die Ansichtsordner sind *Detailstufen*, keine separaten politischen/geografischen Karten.
9. **Kartenmodi filtern sichtbare Inhalte.** Für jeden Modus kann der Nutzer Gruppen und/oder Eigenschaften/Objektkategorien auswählen (z. B. Flüsse, Städte, Landesgrenzen). Die Auswahl gilt über **alle** Detailstufen. Nicht erforderlich: alle fünf/sechs Stufen einzeln konfigurieren. Filter dürfen nicht von sichtbaren Layernamen allein abhängen; strukturierte Kategorien oder stabile IDs verwenden.
10. **Bestehende Bearbeitung erhalten.** Das Layerpanel bleibt für Objektorganisation, manuelles Verstecken, Sperren und Editieren bestehen. Eine Modusleiste in der Kartenoberfläche bietet Auswahl; Moduseinstellungen in einem separaten Dialog/Panel. Auswahl eines Modus darf den aktiven Zeichenlayer nicht heimlich ändern. Versteckte Objekte müssen über einen expliziten Bearbeitungs-/Alle-anzeigen-Weg erreichbar bleiben.
11. **Nicht destruktiv.** Kartenmodi und Fade verändern keine Vektorgeometrie oder gespeicherten Originalfarben. Karteneinstellungen und Metadaten werden mit der Weltkarte persistent gespeichert; gewöhnliche VectorCraft-Dokumente bleiben unverändert.
12. **Optionale spätere Darstellung pro Modus.** Modusabhängige Farben, Linien und Transparenz werden architektonisch ermöglicht, in der ersten Umsetzung aber nur Sichtbarkeitsregeln gebaut.
13. **Chronica später.** Geografische Objekte behalten stabile IDs; Anbindung nach dem tragfähigen Editier-/Datenmodell.

## B. Gemeinsames Verhalten: Sichtbarkeit

Effektive Sichtbarkeit eines Objekts in der Kartenansicht ist die Kombination aus:
- vorhandener manueller Layer-/Objekt-Sichtbarkeit und Sperrlogik;
- aktiver Kartenmodus-Regel für Kategorie, Eigenschaft oder Gruppe;
- ererbtem oder ausdrücklich überschriebenem Zoom-Fade-Bereich;
- gegebenenfalls technischen LOD-/Culling-Entscheidungen (ohne Datenverlust).

**Wichtig:** Überblendende gefüllte Flächen/Konturen dürfen nicht unnötig doppelte Küstenkonturen, helle Säume oder fehlerhafte Zwischenfarben verursachen. Die konkrete Compositing-Regel ist vor der Implementierung mit realistischen Beispielen festzulegen. Umschalten/Zoomen darf Dokumentinhalt und Undo-Historie nicht verändern.

## C. Altbestand und GitHub-Stand (bei Arbeitsbeginn erneut verifizieren)

| Bestandteil | Stand am 2026-11-07 | Konsequenz |
|---|---|---|
| Weltkartenmodell, 2:1, Georeferenz, `file.newWorldMap` | `main`, Phasen 1.1–1.2 | Bestehende Implementierung weiterverwenden |
| Koordinatenpräzision und Weltzoom bis 1.000.000× | `main`, PR #3/#4 | Tests erhalten, weitere Grenzfälle offen |
| Zoom-Fade-Layeroptionen | PR #5 / `feature/aetherys-zoom-fade` | Nicht als bereits in `main` behandeln; prüfen/integreren |
| Räumlicher Index, Tiles/Cache, Maßstab | Draft PR #6 / `feature/aetherys-worldmap-core` | Inkrementell validieren; noch keine vollständige Streaming-Welt |
| Aetherys-Karteninhalt (Küsten, Landmassen, Biome, ohne Berge) | gestapelte PR #7/#8, ggf. #9 | Karten-Datenbranch und App-Code unterscheiden |
| App-weite Kartenmodi, editierbare beliebige Detailstufen, Kategorien | Noch nicht als vollständig implementiert verifiziert | Neues Produkt-Feature |
| 3D-Globus | Geplant | Später |

Relevante PRs: [#5](https://github.com/samuel-alex-ui/vectorcraft/pull/5), [#6](https://github.com/samuel-alex-ui/vectorcraft/pull/6), [#7](https://github.com/samuel-alex-ui/vectorcraft/pull/7), [#8](https://github.com/samuel-alex-ui/vectorcraft/pull/8), [#9](https://github.com/samuel-alex-ui/vectorcraft/pull/9).

**Kartendatei:** Häufig `maps/Aetherys World Map2.vectorcraft` auf den einschlägigen Feature-Branches. Nie durch generierte Rasterbilder ersetzen. Aktuelle Basisdatei und Branch vor einer Migration ausdrücklich ermitteln. Die bereits existierende Layerhierarchie kann davon abweichen; Migration erst nach Backup und Test.

## D. Noch offene Produktentscheidungen (nicht als entschieden implementieren)

1. **Identität über Detailstufen:** Wie verknüpfen wir mehrere Detailzeichnungen derselben Küste/Stadt? Empfehlung: optionale stabile geografische Objekt-ID, keine automatische Rückpropagation.
2. **Objektsemantik:** Welche Kategorien/Eigenschaften braucht ein geografisches Objekt zuerst? Welche Filter gelten für Gruppen vs. Objekttypen? Empfehlung: erweiterbare Typen und benutzerdefinierte Tags statt Layernamen-Heuristik.
3. **Modus-Stile:** Wann und wie Farben/Linienstärken pro Modus überschrieben werden; erst nach Sichtbarkeits-MVP.
4. **Verborgene Objekte bearbeiten:** Expliziter Editier-/Alle-anzeigen-Modus ohne unbemerkte Modusänderung.
5. **Antimeridian und Pole:** Linien, gefüllte Flächen, Löcher und Editieren an der Weltnaht technisch spezifizieren und testen.
6. **Planetenmaßstab:** UI für Radius/Entfernungen; Maßeinheiten und geodätische Berechnungen.
7. **LOD-Compositing:** Regel für gefüllte überblendende Landmassen und unveränderte Gesamtwirkung beim Wechsel.
8. **Persistenz/Kompatibilität:** Schemaentwicklung, alte Dateien, Migration, Undo/Redo und Exportsemantik.

Die Empfehlungen sind keine verbindlichen Produktentscheidungen, bis sie besprochen und bestätigt sind.

## E. Inkrementelle Codex-Arbeitsfolge

**Vor jeder Phase:** vorhandene Dateien, Branch, relevante PRs und Tests kontrollieren. Ein PR je kleinem Schritt, keine ungeprüften Merges in `main`. Tracking über versionierte Einzelaufträge unter [`docs/aetherys-tasks/`](aetherys-tasks/); GitHub Issues sind in diesem Repository derzeit deaktiviert.

| Reihenfolge | Paket | Ergebnis / Abnahme |
|---|---|---|
| 00 | Branch-/Dateistand konsolidieren | Tatsächlichen Basisbranch, abhängige PRs und aktuelle Karten-Datei dokumentieren; keine Datenverluste |
| 01 | Datenmodell für Kartentypen und Identität | Kleine, versionierbare Semantik mit Tests; normaler Editor unverändert |
| 02 | Beliebig viele Detailstufen | Anlegen/Umbenennen/Löschen, Standard-Untergruppen, Persistenz und Migrationstests |
| 03 | Ererbte Zoom-Fade-Regeln | Zoom-Start/Ende von Eltern mit optionalen Ausnahmen; vorhandenes Fade wiederverwenden |
| 04 | Kartenmodus-MVP | Vordefinierte und benutzerdefinierte Modi; Speichern, Auswahl und UI |
| 05 | Sichtbarkeits-Filter | Gruppentypen/Eigenschaften pro Modus; alle Detailstufen, ohne Geometrieänderung |
| 06 | Editorzugriff & Übergangstests | Unsichtbare Objekte erreichbar; Undo, Export, Fade-Überlappungen |
| 07 | Objektidentität & nahtlose LOD | Zusammenhang feiner/grober Geometrie; keine Rückpropagation |
| 08 | Kartennaht, Maße & Skalierung | Antimeridian/Pole, echte Entfernungen nach optionaler Radiuswahl |
| 09 | Großkarten-Performance | Culling, Segment-Indizes, Tile-/Cache-Konzept mit belastbaren Benchmarks |
| 10 | 3D-Vorschau | Nur lesender Globus auf denselben 2D-Geodaten |
| 11 | Spätere Ausbaustufen | Modus-Stile, Historie, Chronica-Links, fortgeschrittene Editierhilfen |

**Codex-Definition of Done pro Paket:** begrenzte Änderung, konkrete Tests (unit/integration + Speichern/Öffnen), Formatierung, Clippy (sofern ausführbar), Testbericht mit offen genannten Lücken, Rückwärtskompatibilität sowie PR mit Verweis auf dieses Dokument. Keine stillen Änderungen an der realen Aetherys-Karte. Bei Abhängigkeiten zu offenen PRs: sauber rebasen statt Features duplizieren.

## F. Architektur-Orientierung

- `crates/worldmap`: Projektion, geografische Hilfen, Maßstabs-/Tile-Grundlagen.
- `crates/doc`: persistente Dokumentmetadaten, Objekte, Layer, Identität und Regeln.
- `crates/engine`: Kommandos und Undo-/Redo-Integration.
- `crates/ui-egui`: Editor- und Modusoberflächen.
- `crates/render`: effektive Sichtbarkeit, Fade, Zeichnen und Culling.
- `crates/format`: native Roundtrip- und Kompatibilitätstests.
- `maps/`: echte Benutzer-Kartendaten, nicht mit UI-Implementierung verwechseln.

## G. Zusammenarbeit

1. Zuerst entsprechende Auftragsdatei unter `docs/aetherys-tasks/` auswählen und offene Entscheidungen klären.
2. Die Auftragsdatei als konkrete Codex-Aufgabe mit Akzeptanzkriterien übergeben; Codex liest vorhandene Dokumentation und die aktuellen Branches selbst.
3. Codex legt eigenen Branch und PR an; Tests und Einschränkungen in den PR.
4. PR und macOS-App prüfen, erst dann mergen und die Auftragsdatei auf `Abgeschlossen` setzen.
5. Nach jedem Schritt dieses Dokument um tatsächlichen Status/Entscheidungen ergänzen.

*Historischer Leitfaden und bereits existierende Detaildokumentationen bleiben erhalten. Dieses Dokument ist die fortlaufende Roadmap, keine Behauptung, dass alle Ziele schon fertig sind.*
