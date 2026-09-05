# Xerv

[![npm](https://img.shields.io/npm/v/xerv.svg)](https://www.npmjs.com/package/xerv)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

**Modular terminal infrastructure for systems, agents, and automation.**

---

## 1. Xerv

Xerv to nowoczesna, terminalowa platforma — miejsce, w którym system monitoringu,
orkiestracji agentów i automatyzacji spotyka się w jednym miejscu. Zamiast
przełączać się między dwoma, trzema, a cztery innymi narzędziami, Xerv
zbiera to, co potrzebujesz, w jednej aplikacji uruchamianej bezpośrednio
w twoim terminalu.

Zbudowany w języku Rust, Xerv jest szybki, niezawodny i pracuje wszędzie,
gdzie działa terminal — na twoim serwerze, laptopie, czy w sesji SSH.

---

## 2. What is Xerv?

Xerv to interfejs terminalowy, który łączy w sobie trzy potrzebne rzeczy:

- **Obserwację** — podgląd na to, co dzieje się w twoim systemie.
- **Interakcję** — możliwość reagowania na to, co widzisz, w czasie rzeczywistym.
- **Automatyzację** — uruchamianie i nadzór nad powtarzalnymi zadaniami.

Pomyślaj o Xerv jako o centrum dowodzenia twoim terminala. To coś więcej niż
narzędzie CLI — to interfejs, który reaguje na ciebie i na to, co się dzieje
wokół.

> Xerv jest obecnie w fazie **pre-alpha**. To oznacza, że budujemy fundament.
> Interfejs, komendy i możliwości będą się rozwijać. To, co dziś widzisz, to
> szkielet — ale już dzisiaj możesz uruchomić go, zobaczyć pierwszą stronę
> i dać nam znać, jak ma to wyglądać.

---

## 3. Why Xerv?

Terminal jest domem wielu programistów, administratorów i inżynierów. Ale
domyślne narzędzia terminala — `top`, `htop`, `ps`, `dmesg` — każde mówi
coś innego. Przełączanie się między nimi kurczy swój czas i uwagę.

Xerv pyta: **dlaczego nie mieć jednego miejsca?**

- Nie musisz zapamiętywać dziesięciu różnych komend.
- Nie musisz przełączać się między panedkami.
- Nie musisz zgadywać, co się stało.

Xerv daje ci **jedną powierzchnię** — czystą, szybką i pod twoją kontrolą.

To, dla kogo Xerv jest przydatny:

| Kto? | Po co? |
|------|--------|
| **Programista** | Szybki podgląd na aplikację, logi, procesy — bez wychodzenia z terminala. |
| **Administrator** | Nadzór nad usługami, reakcja na incydenty — w czasie rzeczywistym. |
| **Inżynier ds. automatyzacji** | Uruchamianie powtarzalnych zadań, monitoring, update’y — z jednego miejsca. |
| **Każdy, kto pracuje w terminalu** | Czytelny interfejs zamiast stosu tekstowych komend. |

---

## 4. What can Xerv do?

Obecnie Xerv oferuje:

- **Interfejs terminalowy (TUI)** — czysty, kolorowy interfejs działający
  bezpośrednio w twoim terminalu. Nie wymaga GUI.
- **Nawigacja klawiaturowa** — sterujesz Xerv jedynie z klawiatury.
- **System komend** — `xerv --version`, `xerv help`, `xerv update` i inne.
- **Automatyczna kontrola wersji** — Xerv wie, jaką masz wersję i potrafi
  samemu się aktualizować.
- **Sprawdzanie aktualizacji** — Xerv sprawdza, czy dostępna jest nowsza wersja,
  i informuje cię o tym bezpośrednio w interfejsie.

Nie ma jeszcze monitoringu procesów, logów ani dashboardów. To dopiero
początek — Xerv ma je dodać.

---

## 5. Features

| Funkcja | Co to daje? | Po co? |
|--------|-------------|--------|
| **Terminal User Interface (TUI)** | Czytelny, kolorowy interfejs w twoim terminalu | Nie musisz wchodzić w struktury tekstowe — wszystko widzisz graficznie |
| **Klawiaturowa nawigacja** | Sterowanie całego interfejsu z klawiatury | Nie musisz zwalaniać myszy — poruszasz się szybko i precyzyjnie |
| **System komend (CLI)** | `xerv --version`, `xerv help`, `xerv update` | Szybki dostęp do najważniejszych działań bez uruchamiania całego TUI |
| **Automatyczna aktualizacja** | Xerv sam sprawdza i pobiera nowe wersje | Zawsze masz najnowsze poprawki i funkcje — bez ręcznej roboty |
| **Wbudowany update z rollbackiem** | Aktualizacja jest atomiczna, a w razie błędu Xerv wraca do poprzedniej wersji | Bezpieczna aktualizacja — nigdy nie zostaniesz z uszkodzoną instalacją |

---

## 6. The Xerv Interface

Xerv uruchamia się jako interfejs terminalowy (TUI). Otwiera się w twoim
oknie terminala — nie musisz niczego uruchamiać osobno.

Interfejs składa się z:

- **Nagłówka** — pokazuje nazwę projektu, wersję i status.
- **Panelu bocznego** — nawigacja po sekcjach.
- **Obszaru głównego** — miejsce, gdzie wyświetlane są dane.
- **Paska poleceń** — wpisz komendę, aby szybko wykonać akcję.

Nawigacja:

| Klawisz | Akcja |
|--------|-------|
| `↑ ↓ ← →` | Przesuwaj się po interfejsie |
| `Enter` | Wejdź w wybraną pozycję |
| `Backspace` | Wróć |
| `q` lub `Esc` | Wyjdź |
| `h` | Otwórz pomoc |
| `U` | Otwórz ekran aktualizacji (gdy dostępna) |

> 📸 **Screenshots i demo będą dostępne wkrótce.** Ta sekcja czeka na materiały
> wizualne, gdy interfejs zostanie ostatecznie dopracowany.

---

## 7. Installation

Xerv jest instalowany jak każdy pakiet npm — bez uprawnień roota:

```bash
npm install -g xerv
```

> Wymaga: Node.js 18+ (dla npm) oraz system Linux lub macOS.

---

## 8. Getting Started

Po zainstalowaniu uruchom Xerv:

```bash
xerv
```

To otworzy interfejs terminalowy. Możesz także użyć szybkich komend:

```bash
xerv           # Uruchom TUI
xerv help      # Pokaż dostępne komendy
xerv version   # Pokaż wersję
```

---

## 9. Updating

Xerv może być aktualizowany na dwa sposoby:

1. **Wewnątrz TUI** — Xerv sam sprawdza dostępność nowej wersji i wyświetla
   powiadomienie. Wystarczy wybrać `Update` i potwierdzić.
2. **Z linii komend**:

```bash
xerv update
```

Pobiera najnowszą wersję, weryfikuje ją i instaluje. W razie problemu
Xerv automatycznie przywraca poprzednią wersję.

> Obecnie system aktualizacji sprawdza wersję na GitHubie i pobiera
> gotowy binarny plik. To działa jako prototyp — w przyszłości będzie
> obsługiwał dodatki i rozszerzenia.

---

## 10. Uninstall

```bash
npm uninstall -g xerv
```

To całkowicie usuwa Xerv i wszystkie powiązane pliki.

---

## 11. Addons / Plugins / Modules

> **[Planned]** System dodatków (addons/plugins/modules) jeszcze nie istnieje.
> Xerv ma go w przyszłości — pozwoli on dodawać nowe widoki, źródła danych
> i automatyzacje bez modyfikacji rdzenia projektu.

Jeśli chcesz pomóc w zaprojektowaniu tego systemu — zerknij do
[Issues](https://github.com/XonofiliusPL/Xerv-Core/issues).

---

## 12. Screenshots / Demo

> Materiały wizualne będą dodane, gdy interfejs zostanie ostatecznie
> dopracowany. Ta sekcja czeka na screenshots i nagrania GIF.

---

## 13. Roadmap

| Co? | Status |
|-----|--------|
| Xerv Core (Rust) — error handling, config, state | ✅ Done |
| Stabline API i wersjonowanie | ✅ Done |
| Agent Workspace (model, drzewo) | ✅ Done |
| TUI — nawigacja, interfejs, identyfikacja wizualna | ✅ Done |
| System aktualizacji — GitHub release, checksum, atomic replace | ✅ Done |
| dystrybucja przez npm (`npm install -g xerv`) | ✅ Done |
| Monitoring procesów i systemu | 🔶 Planned |
| Logi i streamowanie danych w TUI | 🔷 Future |
| System dodatków (addons/plugins/modules) | 🔷 Future |
| Motywy i personalizacja wyglądu | 🔷 Future |

Legend: ✅ = gotowe | 🔶 = planowane | 🔷 = w przyszłości

---

## 14. Project Status

**● Pre-alpha — intensywny rozwój**

Xerv Core jest w trakcie aktywnego rozwoju. To, co dzisiaj widzisz, to
fundament — interfejs, system komend, aktualizacje. API, architektura i
wszystkie szczegóły mogą ulec zmianie.

---

## 15. Links

- **GitHub**: [XonofiliusPL/Xerv-Core](https://github.com/XonofiliusPL/Xerv-Core)
- **npm**: [xerv](https://www.npmjs.com/package/xerv)
- **Issues**: [Zgłoś problem](https://github.com/XonofiliusPL/Xerv-Core/issues)
- **Changelog**: [Commits](https://github.com/XonofiliusPL/Xerv-Core/commits/main)

---

## License

Xerv Core jest przeznaczony do publikacji na licencji **MIT OR Apache-2.0**.

---

*Xerv — zbudowany w Rust. Dla każdego, kto pracuje w terminalu.*
