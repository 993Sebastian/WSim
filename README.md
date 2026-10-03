# WSim

Rundenbasierte Wirtschaftssimulation 1900–2100 für Windows.

- Anforderungen: [`docs/LASTENHEFT.md`](docs/LASTENHEFT.md)
- Architektur und Meilensteine: [`docs/ARCHITEKTUR.md`](docs/ARCHITEKTUR.md)
- Offene Punkte: [`docs/OFFENE_PUNKTE.md`](docs/OFFENE_PUNKTE.md)
- Regeln für die Entwicklung: [`CLAUDE.md`](CLAUDE.md)

## Aufbau

| Ordner | Inhalt |
| --- | --- |
| `crates/wsim-core` | Simulationskern (Rust, ohne Oberfläche) |
| `crates/wsim-data` | Laden und Prüfen der Datendateien |
| `crates/wsim-cli` | Kommandozeile `wsim` für Läufe ohne Oberfläche |
| `app/src-tauri` | Windows-Programm (Tauri-Hülle) |
| `ui/` | Oberfläche (TypeScript + React) |
| `data/` | Spielinhalte und alle Texte |

## Entwickeln

Voraussetzungen: Rust (Version kommt aus `rust-toolchain.toml`), Node.js 22, pnpm 10.
Unter Linux zusätzlich `libwebkit2gtk-4.1-dev` und `libgtk-3-dev`.

```sh
pnpm install                     # Abhängigkeiten der Oberfläche
cargo test                       # Tests des Kerns
pnpm -C ui test                  # Tests der Oberfläche
pnpm -C ui e2e                   # Browser-Tests
pnpm -C app dev                  # Spiel im Entwicklungsmodus starten
pnpm -C app tauri build          # Installer bauen (unter Windows: NSIS-setup.exe)
cargo run -p wsim-cli -- info    # Kommandozeile
```

Den fertigen Windows-Installer baut die CI bei jedem Push; er liegt als Artefakt
`wsim-installer-windows` am jeweiligen Lauf.
