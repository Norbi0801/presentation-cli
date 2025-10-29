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

Podstawowe uruchomienie wymaga wskazania pliku z treścią prezentacji (możesz
też przekazać kilka plików jednocześnie – zostaną odtworzone kolejno):

```bash
cargo run -- presentations/demo.txt
# lub wiele plików
cargo run -- presentations/intro.txt presentations/q_and_a.txt
```

Najważniejsze opcje:

- `--banner <ŚCIEŻKA>` – niestandardowy baner ASCII
- `--title <TYTUŁ>` – nadpisanie tytułu prezentacji
- `--frame-width <LICZBA>` – szerokość ramki prezentacji
- `--theme <neon|amber|arctic>` – wybór jednego z gotowych motywów kolorystycznych
- `--theme-path <ŚCIEŻKA>` – wczytanie motywu z pliku TOML (priorytet nad `--theme`)
- `--instant` – wyłącza animacje (natychmiastowe renderowanie)
- `--skip-banner` – pomija wyświetlenie baneru
- `--playlist <ŚCIEŻKA>` – ładuje listę prezentacji z pliku tekstowego (jedna
  ścieżka na linię, dopuszczalne komentarze `#`)
- `--directory <KATALOG>` – odtwarza wszystkie pliki z katalogu (alfabetycznie)
- `--presenter` – włącza panel prezentera z zegarem i notatkami

Playlisty mają pierwszeństwo po plikach przekazanych bezpośrednio w linii
poleceń, a katalog dopełnia listę. Aplikacja automatycznie pomija duplikaty
utrzymując kolejność pierwszego wystąpienia.

### Format slajdów

Pojedynczy slajd może zawierać wiele wierszy (nagłówki, listy, cytaty itd.).
Pusta linia rozdziela kolejne slajdy, dzięki czemu wygodnie grupujesz treści w
bloki. Notatki prelegenta oznacz prefiksem `@@` – nie pojawią się na ekranie
uczestników, ale będą widoczne w trybie prezentera.

Przykład fragmentu skryptu:

```
# Wstęp
- Czym jest projekt
- Jak działa CLI

@@ przypomnij o ankiecie na końcu
> Cytat inspirujący
```

### Tryb prezentera

Przełącznik `--presenter` uruchamia dodatkowy panel z informacjami o czasie
trwania sesji, numerach slajdów oraz listą notatek oznaczonych w pliku `@@`.
Panel aktualizuje się automatycznie przy każdej zmianie slajdu, a notatki są
numerowane dla łatwej referencji.

### Tryb interaktywny i skróty

Po wczytaniu pierwszej sekwencji prezentacja przechodzi w tryb interaktywny.
Do sterowania użyj następujących skrótów klawiaturowych:

- `←` / `→` (lub `Enter`) – przejście do poprzedniej / następnej sekwencji,
- `+` / `-` – zwiększenie lub zmniejszenie szerokości ramki na bieżącym widoku,
- `q` (lub `Esc`) – zakończenie prezentacji.

Zmiana szerokości ramki działa w locie – bieżąca sekwencja zostanie natychmiast
przerysowana z uwzględnieniem nowego limitu znaków. Dzięki temu możesz szybko
dostosować layout do rozmiaru terminala lub wymagań transmisji.

Jeżeli nie podasz baneru, aplikacja użyje ścieżki określonej w zmiennej
`DEFAULT_BANNER_PATH`.

### Motywy w plikach TOML

Możesz przygotować własny motyw kolorystyczny w pliku TOML i przekazać go
przełącznikiem `--theme-path`. Przykładowy plik znajduje się w katalogu
`themes/nebula.toml`:

```toml
name = "Nebula"
accent = "\x1b[38;5;140m"
dim = "\x1b[38;5;240m"
glow = "\x1b[38;5;219m"
```

Pole `name` jest opcjonalne – jeśli go pominiemy, nazwa motywu zostanie
odczytana z nazwy pliku. Poszczególne pola odpowiadają kodom kolorów ANSI
zastosowanym w prezentacji.
