# Aetherys – Codex-Aufträge

**Stand:** 2026-11-10 · **Status:** AETH-01 bis AETH-06 in `main`; AETH-06 auf macOS abgenommen.

[Masterplan](../aetherys-masterplan.md) · [Älterer Projektleitfaden](../aetherys-project-guide.md)

## Reihenfolge

- [ ] [AETH-00 – Bestandsaufnahme: Branches und Kartendatei](AETH-00.md) – [PR #10](https://github.com/samuel-alex-ui/vectorcraft/pull/10), Review offen
- [ ] [AETH-01 – Geo-Objekttypen und Identität](AETH-01.md) – [Entwurf](../aetherys-semantics-design.md), Schema bestätigt, CI erfolgreich; Render-Performancebefunde und Review offen
- [ ] [AETH-02 – Flexible Detailstufen](AETH-02.md)
- [ ] [AETH-03 – Zoom-Fade-Vererbung](AETH-03.md)
- [ ] [AETH-04 – Kartenmodus-Oberfläche](AETH-04.md)
- [ ] [AETH-05 – Kartenmodus-Filter](AETH-05.md)
- [x] [AETH-06 – Bearbeitung und Übergänge](AETH-06.md) – PR #16 gemergt, alle sechs macOS-Funktionstests bestanden
- [ ] [AETH-07 – LOD-Zusammenhang](AETH-07.md) – implementiert, Review und Nutzerabnahme offen
- [ ] [AETH-08 – Weltnaht, Pole, Maßstab](AETH-08.md)
- [ ] [AETH-09 – Performance großer Karten](AETH-09.md)
- [ ] [AETH-10 – 3D-Globusvorschau](AETH-10.md)
- [ ] [AETH-11 – Spätere Erweiterungen](AETH-11.md)

AETH-02: Verhalten bestätigt; [Implementierung und Prüfbericht](../aetherys-detail-levels.md) auf eigenem, auf AETH-01 gestapeltem Branch. Review/Nutzerabnahme offen.

## So arbeiten wir

1. Nur **einen** Auftrag gleichzeitig an Codex übergeben. Einstieg: **AETH-00** als Bestandsaufnahme.
2. Nach Code-PR und lokaler Prüfung dieses Häkchen setzen und den Status in der Einzeldatei ändern.
3. Vor jeder neuen Phase Branch/PR-Abhängigkeiten prüfen und Entscheidungen mit dem Nutzer abstimmen.
4. Neue Produktbeschlüsse im Masterplan ergänzen, statt alte Dokumente zu ersetzen.

**Hinweis:** GitHub Issues sind im Repository deaktiviert. Diese Markdown-Dateien sind deshalb der versionierte, direkt an Codex übergebbare Aufgaben-Backlog.

## Änderungshistorie

| Datum | Änderung |
|---|---|
| 2026-11-07 | AETH-01-Design verlinkt; AETH-00 bleibt separat in PR #10 zur Prüfung. |
