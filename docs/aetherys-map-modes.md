# AETH-04 – Kartenmodi

Weltkarten zeigen oberhalb der Zeichenfläche eine Modusleiste.
Geografisch, Biome, Politisch, Klima, Wirtschaft und Infrastruktur stehen
beim ersten Öffnen zur Auswahl. Unter **Kartenmodi konfigurieren** lassen
sich Modi anlegen, umbenennen, duplizieren und löschen. Die Auswahl in der
Leiste bleibt beim Zoomen und beim Wechsel des Zeichenlayers erhalten.

Beim Löschen des aktiven Modus wird der verbleibende Modus mit der
kleinsten ID aktiv. Ein Modus muss erhalten bleiben. Änderungen und die
aktive Auswahl werden im Dokument gespeichert und sind rückgängig zu
machen. Ein neuer oder duplizierter Modus steht anschließend in der
Auswahl bereit.

## Datei und Befehle

Die native Karte speichert den Katalog, die aktive ID und den nächsten
freien Zähler unter `map_semantics.modes`. Stabile lokale IDs sind
unabhängig von Anzeigenamen, Artwork und Detailstufen. Bis zu 128 Modi
mit jeweils 256 UTF-8-Bytes im Namen sind möglich. Karten ohne gespeicherten
Katalog verwenden die sechs Startmodi und schreiben das Feld erst bei
einer Änderung. Ältere Semantik-Leser lehnen das neue Feld ab. Normale
Dokumente haben keine Modusleiste.

`map.mode.list` liest die aktive ID und den Katalog. `map.mode.select`
wählt eine ID. `map.mode.edit` verwaltet Einträge mit `action`
(`create`, `rename`, `duplicate`, `delete`), `id` für bestehende Modi
und `name` für Neuanlage, Umbenennung oder Kopie.

## Demonstration

Eine neue Weltkarte anlegen, in der Leiste von **Geografisch** zu **Biome**
wechseln und zoomen. Einen eigenen Modus **Handel** anlegen, ihn auswählen
und duplizieren. Die Karte separat speichern und erneut öffnen. Der aktive
Modus und beide Einträge erscheinen wieder. Die sichtbare Geometrie hängt
weiter von den Ebenen- und Zoomregeln ab. Inhaltsfilter folgen in AETH-05,
abweichende Kartenstile in einer späteren Phase.

## Prüfung am 2026-11-10

Alle zwölf lokalen CI-Schritte bestanden, einschließlich 13.083
Workspace-Tests (14 übersprungen). In der macOS-Testversion blieb der im
Dialog angelegte Modus „Handel“ beim Zoomen und nach Speichern und
Wiederöffnen aktiv. Die macOS-Nutzerabnahme steht aus.

Der Vergleich mit 50.000 Pfaden ergab 54,72 → 54,91 ms beim Speichern und
159,34 → 166,43 ms beim Öffnen (Budget jeweils 300 ms). Rendern bei 400 %
lag bei 15,96 → 16,77 ms (Budget 16 ms). Alle anderen Budgets wurden in
beiden Läufen eingehalten. Der Benchmark verwendet keinen Moduskatalog.
