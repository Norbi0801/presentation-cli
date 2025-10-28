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

```bash
cargo run -- <ścieżka_do_pliku_z_prezentacją> [ścieżka_do_banera]
```

Jeżeli drugi argument nie zostanie podany, aplikacja użyje ścieżki określonej
w zmiennej `DEFAULT_BANNER_PATH`.