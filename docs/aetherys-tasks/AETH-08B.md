# AETH-08B · Dynamische Maßstabsleiste

Die Weltkarten-Zeichenfläche zeigt unten links die lokale Entfernung in Metern
oder Kilometern. Die Leiste verwendet den in den Weltkarten-Einstellungen
gespeicherten Radius. Fehlt er, erscheint ein Hinweis.

`canvas.rs` zeichnet das Overlay mit dem bestehenden Canvas-Painter.
`canvas/scale_bar.rs` verwendet die Projektion und `WorldMap::local_scale`
am Breitengrad der Viewportmitte. Bei gedrehter Ansicht berücksichtigt die
Leiste beide Komponenten der Bildschirmhorizontalen. Sie wählt ein Intervall
von 1, 2 oder 5 mal einer Zehnerpotenz. Das Overlay verarbeitet keine Eingaben
und benötigt keinen zusätzlichen Renderdurchlauf über die Karte.

Die Leiste gibt die lokale Skalierung am Viewportmittelpunkt an. Für
Entfernungen zwischen weit auseinanderliegenden Punkten ist die vorhandene
Großkreis-Messung zuständig. Liegt die Viewportmitte außerhalb der Karte,
wird die Breite auf den nächsten Pol begrenzt. Am Pol selbst fällt der
Ost-West-Maßstab auf null und die ungedrehte Anzeige zeigt einen Hinweis.

## Prüfungen

- Zwei Maßstabstests bestanden. Sie prüfen Kilometer und Meter, den
  gespeicherten Radius, 60° Breite, den Pol und 1.000.000× Zoom.
- Zwei bestehende Globus-Tests, Formatprüfung und Crate-Layerprüfung bestanden.
- Keine bestehenden Kartendateien geändert.
- Native macOS-App gebaut und gestartet. In einer synthetischen Weltkarte
  mit 6.371 km Radius war die Leiste unten links mit `5000 km` sichtbar.
