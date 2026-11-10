# AETH-06: Editiermodus und Render-Übergänge

**Status:** Abgeschlossen. PR #16 ist in `main`; der Nutzer hat alle sechs macOS-Funktionstests erfolgreich durchgeführt.

**Reihenfolge:** 06 / 11  
**Referenz:** [Masterplan](../aetherys-masterplan.md)

## Ziel und Codex-Auftrag

Entwerfe und implementiere expliziten Zugang zu im Modus verborgenen Objekten, ohne normale Layerorganisation zu verlieren. Prüfe überblendende Landfüllungen/Küstenlinien auf Doppelkonturen und Helligkeitsfehler. Undo, Editor-Selektion, Export und persistente Modus-/Fade-Einstellungen testen; Compositing vorher abstimmen.

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
- [x] PR mit reproduzierbarem synthetischem Beispiel
- [x] Nutzerprüfung auf macOS abgeschlossen
- [x] Masterplan/Status aktualisiert


## Bedienung und Übergang

In der Modusleiste **Ausgeblendete Kartenobjekte bearbeiten** aktivieren. Das
zeigt durch Modusregeln oder Zoom ausgeblendete Objekte in der aktuellen
Kartenansicht. Die normale Auswahl und das Layerpanel bleiben verfügbar.
Augensymbol und Sperren gelten weiter. Der Schalter ist pro geöffnetem
Dokument temporär und erzeugt keinen Undo-Schritt. Bearbeitungen lassen sich
wie gewohnt rückgängig machen.

### Synthetisches Beispiel

Zwei benachbarte Detailstufen anlegen. In jeder Stufe einen etwas versetzten
Pfad mit Landfüllung, Küstenstrich und der Kategorie `core:coastline` anlegen.
Die grobe Stufe von 2× bis 4× ausblenden, die feine über dasselbe Intervall
einblenden. Bei 3× bleibt die gemeinsame Landfläche deckend. Die Füllfarben
mischen sich über premultiplizierte RGBA-Werte. Wo nur eine Silhouette Land
enthält, blendet sie weich ein oder aus. Bis zur Mitte ist der grobe
Küstenstrich sichtbar, ab der Mitte der feine. Versetzte Striche können dabei
sichtbar springen. Der Renderer verändert weder Pfade noch gespeicherte Regeln.

Die Paarung gilt für zwei benachbarte, markierte Detailstufen mit
komplementären Regeln und einfachen Küstenpfaden oder zusammengesetzten Pfaden
mit Volltonfarbe. Leere Standardgruppen sind zulässig. Stufen mit gemischter
Grafik, komplexem Aussehen, Masken, lokalen Fade-Ausnahmen, Isolation sowie
Übergangsflächen über 16 Millionen Bildpunkten verwenden den bisherigen Fade.
Der normale Export verwendet weiterhin die gespeicherte manuelle Sichtbarkeit,
ohne Kartenmodusfilter und Zoom-Fade. Es gibt kein geometrisches Morphing und
keine Synchronisierung der Detailstufen. Die Datenbranches aus PR #7–#9
blieben unangetastet.

### Lokale Prüfung

Der Pixeltest prüft an zwei versetzten Küsten die Deckkraft der gemeinsamen
Fläche, den Strichwechsel, die Differenzfläche, beide Endpunkte und den
unveränderten Rasterexport. Der macOS-Test nutzt eine separate synthetische
Datei und prüft Modusfilter, Auswahl, Undo/Redo und Wiederöffnen. Dabei bleibt
der Modus gespeichert, die Bearbeitungsansicht wird zurückgesetzt.

Vier Render-Tests und ein gezielter Editor-Test bestanden. Der Editor-Test
prüft zusätzlich Sperren, manuelles Ausblenden und den nativen Roundtrip.
Formatprüfung, gezieltes Clippy und der native Build wurden ausgeführt.
Die vollständige CI wurde lokal nicht erneut ausgeführt.
