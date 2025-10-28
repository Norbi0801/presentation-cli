## Presentation CLI

Narzędzie pozwala na odtwarzanie prezentacji tekstowych w terminalu. Stałe
kontrolujące wygląd i podstawową konfigurację przechowywane są w pliku `.env`.

### Konfiguracja środowiska

W repozytorium znajduje się przykładowy plik `.env` z następującymi
zmiennymi:

```
FRAME_WIDTH=200
COLOR_ACCENT=\x1b[38;5;208m
COLOR_DIM=\x1b[38;5;94m
COLOR_GLOW=\x1b[38;5;159m
DEFAULT_BANNER_PATH=presentations/banner.txt
PRESENTATION_TITLE=Rust Lab Terminal
```

Możesz je dostosować do potrzeb konkretnej prezentacji, aby zmienić szerokość
ramki, kolorystykę czy tytuł sekcji nagłówkowej. Zmiana `DEFAULT_BANNER_PATH`
pozwala wskazać domyślny baner wyświetlany przed prezentacją.

### Uruchomienie

Aplikacja korzysta z [clap](https://github.com/clap-rs/clap), dzięki czemu
zapewnia czytelne komunikaty o błędach oraz wbudowaną pomoc:

```bash
cargo run -- --help
```

Podstawowe uruchomienie wymaga wskazania pliku z treścią prezentacji:

```bash
cargo run -- presentations/demo.txt
```

Najważniejsze opcje:

- `--banner <ŚCIEŻKA>` – niestandardowy baner ASCII
- `--title <TYTUŁ>` – nadpisanie tytułu prezentacji
- `--frame-width <LICZBA>` – szerokość ramki prezentacji
- `--theme <neon|amber|arctic>` – wybór jednego z gotowych motywów kolorystycznych
- `--instant` – wyłącza animacje (natychmiastowe renderowanie)
- `--skip-banner` – pomija wyświetlenie baneru

Jeżeli nie podasz baneru, aplikacja użyje ścieżki określonej w zmiennej
`DEFAULT_BANNER_PATH`.
