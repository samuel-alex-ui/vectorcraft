# AETH-11: Erweiterungen danach

**Status:** 11A umgesetzt ([PR #25](https://github.com/samuel-alex-ui/vectorcraft/pull/25)). Für 11B fehlt der Schnittstellenvertrag der Chronica-App. Für 11C sind lineare Weltjahre bestätigt.

**Reihenfolge:** 11 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Eigene Darstellungsstile pro Modus (nur Anzeige, Originalfarbe unverändert), Chronica-Links über stabile IDs sowie ggf. historische Zeitstände jeweils separat spezifizieren und als eigene PRs planen, nicht im MVP vorwegnehmen.

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
- [ ] Offene Designfragen entschieden
- [ ] Umsetzung isoliert und rückwärtskompatibel
- [ ] Tests und Format-/Build-Prüfungen dokumentiert
- [ ] PR mit Demonstration oder Beispieldatei
- [ ] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert

## Getrennte Folge-PRs

### 11a · Darstellungsstile pro Kartenmodus

- **Erster Umfang.** Eine optionale Volltonfarbe für vorhandene Füllungen
  einer geografischen Kategorie im aktiven Modus. Die Regel gilt über
  Detailstufen hinweg. Sichtbarkeitsfilter greifen zuerst. Pfade ohne
  Füllung bleiben ungefüllt. Originalfarben und Geometrie bleiben im
  Dokument erhalten. 2D-Ansicht und Globus sollen dieselbe Darstellung
  verwenden. Der bisherige Export behält seine Darstellung.
- **Anschluss.** Die vorhandenen stabilen Modus- und Kategorie-IDs
  verwenden und die Regel im Renderer bei der Farbauswahl anwenden.
  Einen Moduswechsel und Änderungen an Stilregeln getrennt rückgängig
  machen können. Alte Karten enthalten keine Stilregeln.
- **Bestätigte Priorität.** Die erste passende Regel gewinnt. PR #25 baut
  auf dem Globus aus PR #23 auf. Strichfarben, Strichbreiten und komplexe
  Füllungen gehören in spätere Erweiterungen.
- **Abnahme.** Zwei Modi, dieselbe Küste auf zwei Detailstufen.
  Pixelvergleich mit und ohne Regel, Filter und Fade, native
  Speicherung, Undo/Redo und unveränderter Export.

### 11b · Chronica-Verknüpfungen

- **Erster Umfang.** Ein optionaler Verweis pro geografischer Entität.
  Alle Darstellungen mit derselben Entity-UUID zeigen denselben Verweis.
  Abfragen können `map.entityRepresentations` wiederverwenden. Die
  Geometrien bleiben unabhängig. Einträge und Links lassen sich lokal
  bearbeiten und entfernen.
- **Vor Umsetzung entscheiden.** Chronica muss einen stabilen
  Eintragsschlüssel und einen freigegebenen Öffnungsweg liefern.
  Erst danach das persistente Feld und die Zuständigkeit für den Link
  festlegen. Synchronisierung, Hintergrundabrufe und Konten sind
  nicht Teil dieses PRs.
- **Geprüfte Quelle.** Das [Datenschema von Chronica-Data](https://github.com/samuel-alex-ui/Chronica-Data/blob/40b3e24de5c32f62f99be05fdc472d07db449c3c/CAMPFIRE-DATA-SCHEMA.md) dokumentiert unveränderliche `_id`-Werte für Campfire-Exporte. Es schließt die separate Chronica-App ausdrücklich aus. Deren Eintragsschlüssel und Öffnungsweg bleiben offen.
- **Abnahme.** Verweis über zwei Detailstufen finden, löschen und
  wiederherstellen. Fehlende Verweise und ungültige Eingaben prüfen.
  Speichern, Wiederöffnen und Undo/Redo ohne Chronica-Verbindung testen.

### 11c · Historische Zeitstände

- **Erster Umfang.** Nach Festlegung der Weltzeit einen expliziten,
  nur lesenden Zeitpunkt für die Ansicht prüfen. Objektversionen
  behalten die geografische UUID und erhalten getrennte Gültigkeit.
  Den Zeitraumfilter mit Kartenmodus und Detailzoom kombinieren.
- **Bestätigte Zeitbasis.** Lineare Weltjahre. Als erste Umsetzung sind
  ganzzahlige Jahre und halb offene Intervalle vorgesehen (Beginn
  einschließlich, Ende ausschließlich). Fehlende Grenzen lassen das
  Intervall in der jeweiligen Richtung offen. Karten ohne Zeitangaben
  bleiben bei jedem Vorschaujahr sichtbar. Vor der Speicherung noch
  die Behandlung überlappender Versionen festlegen.
- **Abnahme.** Grenzzeitpunkte, nicht datierte Altobjekte und
  unverändertes Original beim Wechsel des Vorschauzeitpunkts.
  Historische Vorschau und Undo-Historie bleiben getrennte Konzepte.

PR #23 für den Globus war bei der Prüfung noch offen. Der Plan
basiert auf `main`; die spätere Globe-Integration von 11a setzt
dessen Merge voraus. Die Datenbranches aus PR #7–#9 bleiben unverändert.
