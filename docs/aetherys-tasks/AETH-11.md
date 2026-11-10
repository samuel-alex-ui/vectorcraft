# AETH-11: Erweiterungen danach

**Status:** Drei getrennte Spezifikationen zur Review. Umsetzung ausstehend.

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
- **Vor Umsetzung entscheiden.** Bei mehreren passenden Kategorien
  explizite Regelpriorität bestätigen. Strichfarben, Strichbreiten
  und komplexe Füllungen später einzeln spezifizieren.
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
- **Abnahme.** Verweis über zwei Detailstufen finden, löschen und
  wiederherstellen. Fehlende Verweise und ungültige Eingaben prüfen.
  Speichern, Wiederöffnen und Undo/Redo ohne Chronica-Verbindung testen.

### 11c · Historische Zeitstände

- **Erster Umfang.** Nach Festlegung der Weltzeit einen expliziten,
  nur lesenden Zeitpunkt für die Ansicht prüfen. Objektversionen
  behalten die geografische UUID und erhalten getrennte Gültigkeit.
  Den Zeitraumfilter mit Kartenmodus und Detailzoom kombinieren.
- **Vor Umsetzung entscheiden.** Kalender oder lineare Zeitachse,
  offene Intervallgrenzen, Überlappungen und das Verhältnis zu Chronica.
  Ohne diese Festlegungen keine Speicherung und keine Historienoberfläche.
- **Abnahme.** Grenzzeitpunkte, nicht datierte Altobjekte und
  unverändertes Original beim Wechsel des Vorschauzeitpunkts.
  Historische Vorschau und Undo-Historie bleiben getrennte Konzepte.

PR #23 für den Globus war bei der Prüfung noch offen. Der Plan
basiert auf `main`; die spätere Globe-Integration von 11a setzt
dessen Merge voraus. Die Datenbranches aus PR #7–#9 bleiben unverändert.
