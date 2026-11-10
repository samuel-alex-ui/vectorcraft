# AETH-05 – Kartenmodusfilter

Die Schaltfläche **Kartenmodusfilter** in der Modusleiste öffnet die
Ausblendregeln für den aktiven Modus. Wähle eine Kategorie, gib einen Tag
oder einen Eigenschaftsschlüssel mit Wert ein oder übernimm im Ebenenpanel
markierte Ebenen und Gruppen. **Anzeigen** entfernt die jeweilige Regel aus
dem Entwurf. **OK** speichert den Entwurf in einem Undo-Schritt.

Ein Treffer genügt, um ein Objekt oder eine Gruppe samt Inhalt auszublenden.
Gruppenregeln verwenden die Dokument-ID der Gruppe. Kategorien verwenden
semantische IDs, die über mehrere Detailstufen hinweg wiederverwendet werden
können. Tags und Eigenschaftswerte werden exakt verglichen. Die manuelle
Sichtbarkeit und der Zoom-Fade gelten weiterhin. Ein Modus ohne Regeln zeigt
alle nach diesen beiden Kriterien sichtbaren Inhalte.

Die native Datei speichert bis zu 128 Regeln pro Modus. Beim Duplizieren
eines Modus werden seine Regeln mitkopiert. Regeln für gelöschte Gruppen
bleiben als entfernbare Einträge im Dialog. Alte Karten ohne Filter öffnen
mit leerem Regelsatz. Ältere Semantik-Leser lehnen das neue Feld beim Öffnen
ab.

## Demonstration

Zwei Objekte in verschiedenen Detailstufen mit derselben Kategorie
`core:river` versehen. Im Modus **Biome** den Kartenmodusfilter öffnen und
**Fluss** ausblenden. Beim Wechsel zu **Geografisch** erscheinen beide
Objekte wieder, sofern sie bei diesem Zoom und nach ihren Ebenenregeln
sichtbar sind. Die Karte separat speichern und erneut öffnen, um den
gespeicherten Modus und seine Regeln zu prüfen.

`map.mode.setFilter` ersetzt die Regeln eines Modus. Beispiel für Modus 2:

```json
{"id":2,"rules":[{"kind":"category","id":"core:river"}]}
```

`map.mode.list` zeigt die Regeln pro Modus. Die Filter wirken in der
Kartenansicht und bei der zoombewussten Canvas-Auswahl. Ausgaben ohne
Kartenansichtszoom verwenden den Dokumentinhalt und seine manuelle
Sichtbarkeit. Der explizite Bearbeitungszugang zu gefilterten Inhalten und
die Exportprüfung folgen in AETH-06.

## Prüfung am 2026-11-10

Alle zwölf lokalen CI-Schritte bestanden. 13.084 Workspace-Tests bestanden
und 14 wurden planmäßig übersprungen. Der Integrationstest deckt die vier
Regeltypen, zwei Detailstufen, Auswahl, manuelle Sichtbarkeit, Zoom-Fade,
Undo/Redo und die native Datei ab. In der macOS-App wurden der Dialog und
der wieder geöffnete Regelsatz einer separaten Karte geprüft. Die
macOS-Nutzerabnahme steht aus.

Im Einzelvergleich mit 50.000 Pfaden lag Rendern bei 400 % in beiden
Ständen über dem Budget von 16 ms (16,46 → 26,78 ms). Speichern und Öffnen
blieben unter 300 ms (55,32 → 82,05 ms und 163,05 → 168,11 ms).
Die Benchmark-Karte verwendet keine aktiven Modusfilter.
