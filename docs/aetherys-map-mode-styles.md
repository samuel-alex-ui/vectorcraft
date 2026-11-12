# AETH-11A – Kartenmodusstile

**Status:** Implementiert, Review offen.

In der Kartenmodusleiste **Kartenmodusstile** öffnen. Eine geografische
Kategorie und ihre Anzeigefarbe wählen. Die Liste wird von oben nach
unten ausgewertet. Mit **Hoch** und **Runter** lässt sich die Priorität
ändern. Die erste Regel für eine Kategorie des Objekts gewinnt. Regeln
gehören zum aktiven Modus und gelten über Detailstufen hinweg.

Die Ansicht färbt vorhandene Volltonfüllungen einfacher Pfade und
zusammengesetzter Pfade. Filter und Zoom-Fade gelten weiter. Beim
Bearbeiten ausgeblendeter Objekte erscheinen die Originalfarben.
Striche, Verläufe, Muster und komplexe Effekte sind außerhalb des
ersten Umfangs. Export und gespeicherte Objektfarben bleiben unverändert.

`map.mode.setStyles` ersetzt die geordnete Liste eines Modus in einem
Undo-Schritt. Eine leere Liste entfernt dessen Stile. Beim Duplizieren
eines Modus werden die Regeln übernommen. Beim Löschen werden sie mit
dem Modus entfernt. Vorhandene Dateien ohne Regeln bleiben kompatibel.
Ältere App-Versionen können Dateien mit den neuen Regeln ablehnen.
