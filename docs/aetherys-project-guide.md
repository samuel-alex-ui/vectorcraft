# Aetherys World Editor — Projektleitfaden

**Stand:** 3. November 2026  
**Status:** Verbindliche Produktziele und bisherige Architekturentscheidungen; technische Umsetzungen sind, soweit ausdrücklich bezeichnet, noch in Planung.  
**Repository:** `samuel-alex-ui/vectorcraft`  
**Lokaler Projektpfad:** `/Users/samuelalex/Developer/GitHub/vectorcraft`  
**Zielablage im Repository:** `docs/aetherys-project-guide.md`

> **Leitidee:** Eine einzige, geografisch zusammenhängende Fantasywelt bearbeiten — von der vollständigen Planetenansicht bis zu einzelnen Häusern und Grundstücken. Die Bearbeitung erfolgt in 2D. Eine separate 3D-Globusansicht dient der Vorschau und Navigation. Keine Sammlung unabhängiger Karten, die man manuell synchronisieren müsste.

## 1. Bereits getroffene Produktentscheidungen

1. **VectorCraft als Basis weiterentwickeln.** Kein Neuentwurf von Grund auf. Bestehende Rust-Workspace-Struktur und Vektorzeichenwerkzeuge nach Möglichkeit erhalten.
2. **2D ist der maßgebliche Editor.** Alle Kontinente, Küsten, Flüsse, Straßen, Städte und Gebäude werden in 2D erstellt und bearbeitet.
3. **Separate 3D-Globusvorschau.** Die 2D-Kartendaten werden zur Betrachtung auf eine drehbare, zoombare Kugel projiziert. Kein direktes 3D-Zeichnen; kein nahtloser Übergang zwischen 2D- und 3D-Editor erforderlich.
4. **Eine zusammenhängende Welt.** Jedes geografische Objekt besitzt einen eindeutig verortbaren Platz auf demselben Planeten. Regionale Detailansichten dürfen nicht zu voneinander losgelösten, inkonsistenten Karten führen.
5. **Zoom bis zur Ebene einzelner Häuser.** Ziel sind auch Straßenzüge, Grundstücke, Gebäudeumrisse und später bei Bedarf Gebäudegrundrisse; nicht nur Kontinente und Städte.
6. **Vektoren bleiben die geometrische Quelle der Wahrheit.** Keine redundante, dauerhaft auseinanderlaufende zweite Geometriesammlung für dieselben Objekte.
7. **Zunächst equirektanguläre Weltkarte (2:1).** Geografische Breite −90° bis +90°, Länge −180° bis +180°. Der Planet kann eine frei konfigurierbare Größe bekommen; Erdgröße ist nur ein Beispiel, keine feste Vorgabe.
8. **Chronica-Integration erst später.** Künftige Verknüpfung über stabile Objekt-IDs; keine aktuelle Synchronisationsaufgabe.
9. **Modulare, testbare Entwicklung.** Codex setzt kleine, überprüfbare Phasen auf Feature-Branches um; Änderungen gehen nach Prüfung per Pull Request in `main`.

## 2. Was heute funktioniert (Phase 1.1 und 1.2)

- Das Fork-Repository `samuel-alex-ui/vectorcraft` ist lokal auf dem Mac eingerichtet.
- Die Anwendung wurde auf macOS mit Cargo gebaut und gestartet.
- `File → New World Map…` legt eine Weltkarte mit einer **1000 × 500 Punkt** großen 2D-Zeichenfläche an.
- Bestehende Vektorwerkzeuge können Küstenlinien und andere Formen zeichnen.
- Ein eigenständiges Rust-Modul unter `crates/worldmap` enthält die anfängliche Umrechnung zwischen Karten- und geografischen Koordinaten.
- **Phase 1.2:** Ein Weltkarten-Dokument besitzt explizite, validierte und speicherbare Metadaten (Projektion, Kartenausdehnung, optionaler Planetenradius); gewöhnliche VectorCraft-Dokumente bleiben davon unberührt.
- Der Benutzer hat eine eigene einfache Weltkarte gezeichnet, gespeichert, VectorCraft geschlossen und wieder geöffnet. **Dieser praktische Speichern-/Öffnen-Test hat funktioniert.**
- Codex meldete für Phase 1.2 erfolgreich durchlaufene umfangreiche CI-/Rust-Tests. Einzelheiten: [Pull Request #2](https://github.com/samuel-alex-ui/vectorcraft/pull/2), Commit `40cef3b4c`.

**Wichtige Unterscheidung:** Die Kartengröße 1000 × 500 ist eine **geografische Referenzfläche in Dokumentpunkten**, keine Bildauflösung und keine Begrenzung auf 1000 × 500 Pixel. Das allein garantiert aber **keinen** extremen Zoom, ausreichende Koordinatenpräzision oder performantes Zeichnen von Millionen Objekten.

## 3. Zielspektrum des geografischen Zooms

Orientierungswerte für einen erdgroßen Beispielplaneten (Äquatorumfang ca. 40.000 km):

| Betrachtung | Ungefähr sichtbare Breite | Relativer Zoom gegenüber Gesamtwelt |
|---|---:|---:|
| Planet | 40.000 km | 1× |
| Kontinent | 5.000 km | 8× |
| Land/Region | 500 km | 80× |
| Stadt | 10 km | 4.000× |
| Stadtviertel | 1 km | 40.000× |
| Gebäudeumfeld | 100 m | 400.000× |
| Einzelnes Haus und Grundstück | einige zehn Meter | entsprechend stärker |

Diese Faktoren sind **beispielhafte Zielgrößen**, keine bestätigten Fähigkeiten der aktuellen VectorCraft-Engine. Eine frei einstellbare Planetengröße muss berücksichtigt werden.

### Inhaltliche Ebenen / Detailstufen

- **Planet:** Kontinente, Ozeane, große Inseln, Hauptgebirge.
- **Kontinent / Land:** Grenzen, Flusssysteme, Wälder, Regionen, Siedlungsmarkierungen.
- **Stadt:** Stadtgrenze, Stadtmauern, Viertel, Straßen, Plätze, Gewässer.
- **Stadtviertel:** Straßenzüge, Blöcke, Parzellen und Gebäudeumrisse.
- **Haus:** einzelnes Gebäude, Grundstück und ggf. weiter detaillierte Grundrisse (später separat spezifizieren).

Die Darstellung darf je nach Zoom umschalten, vereinfachen oder weitere Details einblenden; sie soll dabei immer eindeutig derselben Welt und denselben geografischen Referenzen zugeordnet bleiben. Unterschiedliche geometrische Repräsentationen derselben Sache können für Generalisierung erforderlich sein, müssen aber konsistent miteinander verknüpft sein.

## 4. Technische Architekturprinzipien

### Geografische Referenz

- Stabiler Weltbezug unabhängig von Fenstergröße, Zoomfaktor, Bildschirmauflösung und aktuell sichtbarem Bereich.
- Klare Transformationen zwischen geografischen Koordinaten, Kartenpunkten und Bildschirm-/Kamerakoordinaten.
- Bei sehr großem Zoom **numerische Präzision ausdrücklich nachweisen**, statt sie von der Vektorbasiertheit abzuleiten. Prüfen: `f64`-Koordinaten, lokale Ursprünge, Kamera-relative Transformation, Clipping, Hit-Testing, Snap, Pfadbearbeitung, Renderer und GPU-Präzision.
- Keine stillschweigende Veränderung geografischer Koordinaten beim gewöhnlichen Zoomen, Verschieben oder Bearbeiten von Ansichtseinstellungen.
- Karte bleibt in der Anfangsphase equirektangulär; mögliche zukünftige Projektionen dürfen nicht implizit als vorhanden gelten.

### Datenmodell / Kartenobjekte

- Vorhandene VectorCraft-Vektorpfade grundsätzlich wiederverwenden.
- Geografische Objektkategorien aufbauend auf den vorhandenen Objekten: z. B. Landfläche/Kontinent, See/Ozean, Fluss, Grenze, Straße, Siedlung/Stadt, Grundstück und Gebäude.
- Logische Ebenen/Gruppen für Organisation und ein-/ausblendbare Inhalte, ohne das normale VectorCraft-Dokumentmodell unnötig zu ersetzen.
- Stabile Objekt-IDs als Grundlage für spätere Beziehungen zu Chronica.
- Sichtbarkeitsregeln nach Maßstab und klare Trennung von Daten, Bearbeitung und Darstellung.

### Große Datenmengen / Performance

- Räumliche Indizierung, Sichtbereichsabfragen und bedarfsgerechtes Laden/Culling evaluieren.
- Hierarchische Detailstufen (LOD) und Kartenkacheln/Chunks als spätere technische Bausteine vorsehen, **nicht** ohne Untersuchung vorschnell implementieren.
- Rendering und Auswahl sollten nur relevante Objekte bearbeiten, nicht auf jedem Frame die gesamte Welt.
- Belastungstests mit realistischen Objektzahlen, extremem Zoom und großen Koordinatenbereichen definieren.

### Antimeridian und Pole

- Die linke und rechte Kante der 2D-Weltkarte repräsentieren denselben geografischen Meridian (−180°/+180°).
- Objekte, die diese Kante kreuzen, müssen in 2D sinnvoll editierbar und in 3D ohne sichtbaren Bruch darstellbar sein.
- Pole und die Verzerrung der equirektangulären Projektion benötigen explizite Regeln; keinesfalls als bereits gelöst behandeln.
- Frühzeitig Tests für um den Kartenrand verlaufende Linien, Polnähe und relevante Auswahl-/Bearbeitungsvorgänge planen.

### 3D-Vorschau

- Spätere eigenständige 3D-Vorschau verwendet dieselbe geografische Grundlage wie der 2D-Editor.
- Erste 3D-Version: drehbarer und zoombarer Globus mit Darstellung der 2D-Weltkarte; später maßstabsabhängige Detaildaten.
- Zunächst **kein** 3D-Bearbeitungswerkzeug, kein nahtloser 2D/3D-Übergang, keine physikalischen Terrain-/Höhenmodelle voraussetzen.

## 5. Entwicklungsreihenfolge (Plan, Änderungen nach Prüfung erlaubt)

### Abgeschlossen

- **1.1:** Menüeintrag und 2:1-Weltkartendokument.
- **1.2:** Persistente geografische Dokumentmetadaten, Speichern/Wiederöffnen, normale Dokumentkompatibilität.

### Als Nächstes – Phase 1.3: Maßstabs- und Koordinatenfundament

**Zuerst die bestehende Architektur analysieren, dann eng begrenzte Änderungen implementieren.**

1. Dokument-, Kamera-, View-, Transformations-, Hit-Test- und Renderer-Code auf reale Zoomgrenzen untersuchen.
2. Konkrete Genauigkeitsziele aus dem Anwendungsfall „erdgroßer Planet → einzelne Häuser“ ableiten; durch Tests nachvollziehbar machen.
3. Sicherstellen, dass die geografische Zuordnung beim Zoomen und Verschieben stabil bleibt.
4. Eine modulare technische Strategie für lokalen Bearbeitungsursprung/Präzision, detaillierte Objektkoordinaten und künftig räumliche Indizes dokumentieren.
5. Grenzfälle für Kartenrand/Polnähe definieren und mit Tests absichern, soweit in dieser Phase sinnvoll.
6. Bestehende VectorCraft-Werkzeuge nicht unnötig umbauen; native Dokumentkompatibilität erhalten.
7. Zum Abschluss benennen, was wirklich implementiert wurde, was nur konzeptionell geplant ist und welche Zoomtests tatsächlich erfolgreich sind.

**Nicht in Phase 1.3 erforderlich:** vollständige Chunk-/LOD-Engine, Tausende echte Städte, 3D-Globus, Chronica-Synchronisation oder eine komplett neue UI.

### Danach

- **1.4 (ggf. in Teilphasen):** Geografische Objektkonventionen, Ebenen und Sichtbarkeitsregeln; sauberes Verhalten am Kartenrand.
- **1.5:** Erste einfache interaktive 3D-Globusvorschau, falls das Koordinaten- und Datenmodell ausreichend stabil ist.
- **Spätere Phasen:** Hochauflösende Stadt-/Gebäudeansichten, räumliche Indizes, Chunk-/LOD-Streaming, sehr große Datenmengen, erweiterte 3D-Detailansichten, Chronica-Anbindung.

Die Nummerierung ist ein Vorschlag und darf anhand der Implementierungsrisiken angepasst werden. Entscheidend ist: **Hausgenauer Zoom muss vor dem Ausbau der Ebenen-/Objektfunktionen als Systemanforderung berücksichtigt werden.**

## 6. Arbeitsweise mit Codex und GitHub

- **Eine Aufgabe = ein überschaubarer Entwicklungsbranch und ein prüfbarer Pull Request.**
- Codex soll vorhandene Dokumentation sowie den tatsächlichen Quellcode lesen, bevor es Architekturentscheidungen trifft.
- Wichtige Designentscheidungen in `docs/aetherys-world-map.md` und/oder diesem Leitfaden aktualisieren.
- Relevante Tests, Formatierung, Clippy und Speichern/Wiederöffnen ausführen, soweit die Umgebung das zulässt. Fehlgeschlagene Tests offen berichten.
- `main` nur nach Prüfung aktualisieren; lokale App nach dem Merge neu bauen (`cargo run --release -p vectorcraft`).
- **Keine Architekturbehauptung als umgesetzt kennzeichnen, solange keine passende Implementierung/Prüfung vorliegt.**

## 7. Referenzen und Übergabe an neue Chats

- Repository: https://github.com/samuel-alex-ui/vectorcraft
- Phase-1.2-Pull-Request: https://github.com/samuel-alex-ui/vectorcraft/pull/2
- Vorhandene technische Dokumentation im Repository: `docs/aetherys-world-map.md`
- **Dieses Dokument:** `docs/aetherys-project-guide.md` (nach dem Einfügen und Committen ins Repository)

**Anweisung für Codex und zukünftige Assistenten:** Lies zuerst `docs/aetherys-project-guide.md` und `docs/aetherys-world-map.md`, prüfe danach die aktuelle Implementierung und den Branchzustand. Behandle die Produktentscheidungen als Ziele, nicht als Beweis, dass entsprechende Funktionen bereits implementiert sind. Wenn ein vorgeschlagener Implementierungsschritt diese Ziele gefährdet, erkläre den Konflikt, bevor er umgesetzt wird.

---

*Dieser Leitfaden ist bewusst als Markdown verfasst, damit er versioniert, erweitert, in GitHub gelesen und Codex als Projektkontext gegeben werden kann.*


## Technischer Nachweis Phase 1.3 (3. November 2026)

Die bestehenden Zoom- und Koordinatenpfade sind im
[technischen Prüfbericht](aetherys-zoom-precision.md) beschrieben. Die kleine
Präzisionskorrektur an der Bildschirmumrechnung hebt die bestehende UI-Grenze
von 640× (Print-Size-Preset-Ausnahme: 853⅓×) nicht an. Hohe synthetische Testfaktoren sind kein Nachweis für vollständig
hausgenaues interaktives Bearbeiten. Die oben festgelegten langfristigen
Produktziele und die dafür noch erforderlichen Maßnahmen bleiben bestehen.


## Technischer Stand Phase 1.4 (3. November 2026)

Weltkarten-Navigation erlaubt nun bis 1.000.000×; normale Dokumente behalten
ihre bisherigen Grenzen. Ein gezielter Rechteck-Werkzeugtest deckt Zeichnen,
Auswählen, Verschieben und Undo ab. Das ist kein vollständiger Nachweis für
alle Werkzeuge, Effekte oder große Szenen. Fit-Befehle bleiben vorerst begrenzt.
Der [Prüfbericht](aetherys-zoom-precision.md) beschreibt Einschränkungen und den
lokalen App-Update-Befehl `bash packaging/macos/aetherys-app.sh --build`.
Die langfristigen Produktziele bleiben unverändert.
