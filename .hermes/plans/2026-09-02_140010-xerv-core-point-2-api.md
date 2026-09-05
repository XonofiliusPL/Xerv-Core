# Plan: Xerv — Punkt 2 (Xerv Core API)

## Goal

Zaprojektować **wyłącznie Punkt 2 — publiczne API** `xerv-core` jako stabilną, niezależną od implementacji warstwę graniczną (Core ↔ przyszłe TUI / CLI / moduły / pluginy / adaptery), bez implementowania tych konsumentów. Kontrakt obejmuje nazwy, typy, sygnatury, warianty błędów, lifecycle, konfigurację, stan oraz **minimalną** podstawę rozszerzalności potrzebną na tym etapie.

---

## Current context / assumptions

- **Punkt 1 ukończony** — `xerv-core` istnieje jako biblioteka z publicznymi typami: `Error`, `Result`, `API_VERSION`, `api_version`, `CoreConfig`, `CoreState`, `XervCore`. Commit `4977b48` (`feat(core): XervCore lifecycle with state persistence`) zamyka Punkt 1.
- **Stan faktyczny API (po Punkcie 1)** — na podstawie `src/lib.rs`:
  - `pub mod config;`, `pub mod core;`, `pub mod error;`, `pub mod logging;`, `pub mod state;`, `pub mod version;` — moduły **publiczne**, więc wszystkie ich elementy `pub` są dziś publiczne.
  - `pub use` eksportuje na szczycie: `CoreConfig`, `XervCore`, `Error`, `Result`, `CoreState`, `API_VERSION`, `api_version`.
  - `XervCore::new(config, state_path) -> Result<Self>`, `config()`, `state()`, `api_version()`, `shutdown() -> Result<()>`.
  - `CoreConfig::load(path) -> Result<Self>`, `Default` z `data_dir` z `directories::ProjectDirs::from("xerv", "xerv", "xerv")` (fallback `./.xerv`), `log_level`, `state_filename`.
  - `CoreState` ma `schema_version: u32`, `started_at_unix: u64`, `boot_count: u64`, `fresh()`, `load(path)`, `save(path)`.
  - Brak `Cargo.lock` w commicie (gitignore) — decyzja archiwalna.
- **Brak zatwierdzonego design docu** dla Punktu 2 (sprawdziłem: `~/.hermes/sessions`, `~/.hermes/memories`, brak wpisów). Wszystkie decyzje poniżej są **propozycjami** wymagającymi potwierdzenia.
- **Nie istnieją** jeszcze: TUI, CLI, daemon, pluginy, agenci, moduły, integracje, VPS, xerv.pl, registry. API Punktu 2 **nie może** zakładać ich konkretnych kształtów.
- Komunikacja z użytkownikiem wyłącznie po polsku (zgodnie z `~/Work/AGENTS.md`).
- Punkt 2 = **tylko stabilizacja kontraktu publicznego** `xerv-core` (samego API), **bez** pisania konsumentów (TUI/CLI/moduły) i **bez** dodawania funkcjonalności.
- Plan zakłada **rozszerzenie minimalne** — wariant (a) w decyzji nr 6. Warianty (b)/(c) zostawiam jako propozycje do późniejszej decyzji, bo plan Punktu 2 nie powinien ich implementować.

---

## Architecture / proposed approach (PROPOZYCJA — do zatwierdzenia)

`xerv-core` po Punkcie 2 udostępnia **dwa poziomy publicznego API**: (1) **stabilny** `xerv_core::api` z minimalnymi aliasami typów i **kontraktem** (które pola publiczne, które gwarantujemy), (2) **warstwę istniejącą** (`XervCore`, `CoreConfig`, `CoreState`, `Error`, `Result`, `API_VERSION`) z drobnymi poprawkami kompatybilności. **Nie** dodajemy żadnych traitów rozszerzeń, rejestrów pluginów, subskrypcji zdarzeń, FSM-ów, builderów, async. **Jedyna** nowa „rozszerzalność" w Punkcie 2 to: (i) moduł `api` jako stabilny reexport, (ii) jawne oznaczenie „experimental" / „frozen" w doc-komentarzach, (iii) **opcjonalne** (do decyzji) `ApiError` jako spójna wariancja publiczna. Przyszłe warstwy (TUI/CLI) **same** zdecydują, jak z API korzystać; Rdzeń nie przygotowuje dla nich haków.

**Granica rozszerzenia (minimalny kontrakt Punktu 2):**
- Rdzeń definiuje **tylko** swój własny publiczny kształt i gwarancje.
- **Nie** udostępnia: `CoreExtension` (trait), `register`, `subscribe`, `ModuleLoader`, `Adapter`, `Provider`, `Dispatcher`, `Context` jako typ generyczny.
- Jeśli przyszły punkt (TUI, CLI, moduły) potrzebuje haka, dostanie go **w swoim** punkcie, nie w Punkcie 2.

---

## Scope of Punkt 2

**Wchodzi:**
1. Moduł `xerv_core::api` z publicznymi reexportami stabilnego API + doc-komentarzem opisującym **kontrakt** (co gwarantujemy, czego nie).
2. Jawne oznaczenie statusu stabilności: `pub mod api { /* @stable */ }`, `pub mod core { /* @frozen w Punkt 2 */ }` w doc-komentarzach.
3. **Opcjonalne** (do decyzji użytkownika): alias `pub type ApiError = Error;` i `pub type ApiResult<T> = Result<T>;` w `api` — dla przyszłych konsumentów, którzy chcą jawnego importu z `api` zamiast z korzenia.
4. `XervCore::is_shutdown() -> bool` — akcesor tylko-do-odczytu dla stanu lifecycle (testowalny, użyteczny w TUI do szarych przycisków).
5. Sprawdzenie i (jeśli brakuje) dodanie `Debug` na `XervCore` (potrzebne w panikach i dev-testach).
6. Testy: (i) że `api::*` wskazuje na te same typy co korzeń, (ii) że `is_shutdown` zwraca poprawne wartości, (iii) że `Debug` się kompiluje.
7. Walidacja końcowa: build, test, clippy `-D warnings`, fmt.

**Nie wchodzi (świadoma decyzja):**
- żaden nowy `trait` (`CoreExtension`, `Adapter`, `Provider`, …);
- żadne `register` / `subscribe` / `Vec<dyn …>`;
- żaden builder ani fluent API;
- żadna binarka (`bin/`), `examples/`;
- żadna integracja z innymi crate'ami (TUI, CLI, daemon, agenci, sieć);
- żadne async, kanały, observer pattern;
- żadne migracje stanu / wersjonowanie schematu;
- żadne nowe zależności runtime/dev;
- żadne breaking changes w istniejącym API (Punkt 1 nie łamie się);
- żadna dokumentacja wykraczająca poza doc-komentarze (`README`, `book/`, `docs/`);
- żadne `cargo doc --open`-targetowane dodatkowe strony.

---

## Decyzje wymagające potwierdzenia użytkownika PRZED implementacją

Każda decyzja binarna. Bez tych odpowiedzi plan nie przejdzie w implementację.

1. **Czy moduł `xerv_core::api` w ogóle powstaje w Punkcie 2?**
   - (a) TAK — moduł `api` z reexportami + doc-kontrakt (propozycja).
   - (b) NIE — Punkt 2 to **wyłącznie** drobne poprawki (is_shutdown, Debug) na istniejących typach, bez nowego modułu.

2. **Aliasy `ApiError` / `ApiResult` w `api`?**
   - (a) TAK — `pub type ApiError = Error;` + `pub type ApiResult<T> = Result<T>;` w `api`.
   - (b) NIE — tylko reexporty typów bez aliasów.

3. **Oznaczenia stabilności:** (b) NIE — standardowe `//! …` bez metatagów `@stable` / `@frozen`.

4. **`XervCore::is_shutdown() -> bool`?**
   - (a) TAK — dodajemy akcesor (propozycja).
   - (b) NIE — konsumenci sami trzymają flagę.

5. **`#[derive(Debug)]` na `XervCore`?**
   - (a) TAK — dodajemy (propozycja).
   - (b) NIE — zostawiamy jak jest (bez `Debug`).

6. **Podstawa rozszerzalności w Punkcie 2 — wariant:**
   - (a) nic (propozycja) — żadnych traitów/rejestrów, minimalny kontrakt;
   - (b) **`pub trait CoreApi { fn core(&self) -> &XervCore; }`** — pusty trait-aspekt, tylko dla czytelności granicy;
   - (c) **pusty `pub mod extensions { pub trait CoreExtension {} }`** — szkielet bez implementacji.

7. **`Cargo.lock` w repo (gitignore)?**
   - (a) NIE — zostawiamy jak jest (zgodnie z Punktem 1, plan i tak nie zmienia).
   - (b) TAK — dorzucamy `Cargo.lock` do repo (mimo workspace bibliotecznego).

8. **Zmiana wersji API (`API_VERSION`)?**
   - (a) NIE — zostaje `0.1.0` (zgodnie z Punktem 1).
   - (b) TAK — bump do `0.2.0` (nowe API = nowa wersja, mimo że „prawie pusty" Punkt 2).

9. **`api_version()` w `XervCore` zwraca przez wartość (`Version`)?**
   - (a) TAK — zostaje jak w Punkcie 1 (sygnatura `pub fn api_version(&self) -> Version`).
   - (b) NIE — zmieniamy na `&'static Version` (wymaga wewnętrznego statycznego buforu — drobna zmiana, ale potencjalnie łamie konsumentów Punktu 1, których jeszcze nie ma).

10. **Format doc-komentarzy:**
    - (a) standardowe `///` Rustdoc, sekcje `# Examples` (propozycja).
    - (b) rozszerzone o `# Stability` z nazwą statusu.

---

## Step-by-step tasks

Każde zadanie: 2-5 min, TDD, kod kopiowalny 1:1, dokładne komendy weryfikacyjne, commit po każdym zielonym teście. Przed rozpoczęciem zakładam, że decyzje 1, 3, 4, 5 = wariant domyślny (a), decyzje 2, 6, 7, 8, 9, 10 = wariant domyślny. Jeśli użytkownik wskaże inaczej — kroki ulegną modyfikacji przed commitem.

### Etap 0: Przygotowanie (bez commita)

**Task 0.1** — Sprawdź stan repo i aktualną wersję.

```bash
cd ~/Projects/xerv
git status
git log --oneline -5
cargo --version && rustc --version
```

Weryfikacja: `cargo --version` → np. `cargo 1.98.0`; `git status` → `nothing to commit, working tree clean`.

**Task 0.2** — Upewnij się, że Punkt 1 jest nienaruszony (sanity test).

```bash
cd ~/Projects/xerv
cargo test -p xerv-core 2>&1 | tail -3
```

Weryfikacja: `test result: ok. 16 passed; 0 failed` (z Punktu 1).

---

### Etap 1: Moduł `api` (jeśli decyzja 1 = a)

**Task 1.1** — Dodaj pusty moduł `api` z testem istnienia.

Stwórz `/home/xono/Projects/xerv/crates/xerv-core/src/api.rs`:

```rust
//! Xerv Core — stabilne publiczne API.
//!
//! Ten moduł jest **kontraktem** między Rdzeniem a przyszłymi warstwami
//! (TUI, CLI, moduły, pluginy, adaptery). Gwarantujemy:
//!
//! - nazwy typów i funkcji wymienione w [`Api`] (aliasy i reexporty);
//! - semantykę metod udokumentowaną w `///` przy każdym typie;
//! - że `Cargo.toml` nie zmienia się w sposób łamiący API bez bumpu
//!   `API_VERSION` w [`crate::API_VERSION`].
//!
//! **Nie gwarantujemy** (są poza zakresem Punktu 2):
//! - żadnych traitów rozszerzeń;
//! - żadnego rejestru pluginów;
//! - żadnych subskrypcji zdarzeń;
//! - żadnego API specyficznego dla konsumenta (TUI/CLI/moduły).

/// Alias stabilnego [`crate::Error`].
pub type ApiError = crate::Error;

/// Alias stabilnego [`crate::Result`].
pub type ApiResult<T> = crate::Result<T>;

// Reexporty stabilnych typów Rdzenia.
pub use crate::config::CoreConfig;
pub use crate::core::XervCore;
pub use crate::error::{Error, Result};
pub use crate::state::CoreState;
pub use crate::version::{api_version, API_VERSION};
```

W `src/lib.rs` dodaj (kolejność alfabetyczna, api przed config):

```rust
pub mod api;
pub mod config;
pub mod core;
pub mod error;
pub mod logging;
pub mod state;
pub mod version;
```

**Task 1.2** — Test reexportów (typ-tożsamość, nie tylko nazwa).

Stwórz `/home/xono/Projects/xerv/crates/xerv-core/tests/api.rs`:

```rust
use std::any::TypeId;

use xerv_core::api::{
    api_version, ApiError, ApiResult, CoreConfig, CoreState, Error, Result, XervCore,
    API_VERSION,
};
use xerv_core::{config, core, error, state, version};

fn type_id_of<T: 'static>(_: &T) -> TypeId {
    TypeId::of::<T>()
}

#[test]
fn api_reexports_match_crate_root_types() {
    assert_eq!(type_id_of(&CoreConfig), type_id_of(&config::CoreConfig));
    assert_eq!(type_id_of(&XervCore), type_id_of(&core::XervCore));
    assert_eq!(type_id_of(&Error), type_id_of(&error::Error));
    assert_eq!(type_id_of(&CoreState), type_id_of(&state::CoreState));
}

#[test]
fn api_aliases_match_crate_root_aliases() {
    // TypeId nie działa na type-aliasach, więc sprawdzamy przez konwersję.
    let e: ApiError = ApiError::Config("x".into());
    let e2: Error = e;
    assert_eq!(e2.to_string(), "config: x");
    let r: ApiResult<i32> = Ok(7);
    let r2: Result<i32> = r;
    assert_eq!(r2.unwrap(), 7);
}

#[test]
fn api_version_constants_match() {
    assert_eq!(api_version().major, API_VERSION.major);
    assert_eq!(api_version().minor, API_VERSION.minor);
    assert_eq!(api_version().patch, API_VERSION.patch);
}
```

Weryfikacja: `cargo test -p xerv-core --test api` → `test result: ok. 3 passed; 0 failed`.

Commit: `feat(core): stable api module with reexports and aliases`.

---

### Etap 2: `XervCore::is_shutdown()` (jeśli decyzja 4 = a)

**Task 2.1** — Dodaj akcesor + test.

W `src/core.rs` dodaj metodę publiczną (np. zaraz po `api_version`):

```rust
    /// Zwraca `true` jeśli `shutdown()` został już wywołany.
    pub fn is_shutdown(&self) -> bool {
        *self.shutdown_flag.lock().unwrap()
    }
```

Test w `/home/xono/Projects/xerv/crates/xerv-core/tests/is_shutdown.rs`:

```rust
use xerv_core::{CoreConfig, XervCore};

fn core(dir: &std::path::Path) -> XervCore {
    let cfg = CoreConfig {
        data_dir: dir.to_path_buf(),
        log_level: "warn".into(),
        state_filename: "s.json".into(),
    };
    XervCore::new(cfg, dir.join("s.json")).unwrap()
}

#[test]
fn fresh_core_is_not_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = core(dir.path());
    assert!(!c.is_shutdown());
}

#[test]
fn after_shutdown_is_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = core(dir.path());
    c.shutdown().unwrap();
    assert!(c.is_shutdown());
}

#[test]
fn double_shutdown_still_shut_down() {
    let dir = tempfile::tempdir().unwrap();
    let c = core(dir.path());
    c.shutdown().unwrap();
    let _ = c.shutdown(); // błąd, ale flaga zostaje
    assert!(c.is_shutdown());
}
```

Weryfikacja: `cargo test -p xerv-core --test is_shutdown` → `test result: ok. 3 passed; 0 failed`.

Commit: `feat(core): XervCore::is_shutdown accessor`.

---

### Etap 3: `#[derive(Debug)]` na `XervCore` (jeśli decyzja 5 = a)

**Task 3.1** — Dodaj `Debug`.

W `src/core.rs`, na deklaracji `pub struct XervCore`, dodaj `Debug`:

```rust
#[derive(Debug)]
pub struct XervCore {
    config: CoreConfig,
    state: Mutex<CoreState>,
    state_path: PathBuf,
    shutdown_flag: Mutex<bool>,
}
```

**Task 3.2** — Test, że `Debug` się kompiluje.

W `/home/xono/Projects/xerv/crates/xerv-core/tests/debug_impl.rs`:

```rust
use xerv_core::{CoreConfig, XervCore};

#[test]
fn xerv_core_implements_debug() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = CoreConfig {
        data_dir: dir.path().to_path_buf(),
        log_level: "warn".into(),
        state_filename: "s.json".into(),
    };
    let c = XervCore::new(cfg, dir.path().join("s.json")).unwrap();
    let s = format!("{c:?}");
    assert!(s.contains("XervCore"));
}
```

Weryfikacja: `cargo test -p xerv-core --test debug_impl` → `test result: ok. 1 passed; 0 failed`.

Commit: `feat(core): derive Debug on XervCore`.

---

### Etap 4: Test „freeze" — gwarancje kontraktu API

**Task 4.1** — Test, że publiczne typy istnieją w obu miejscach (`api` + korzeń).

Stwórz `/home/xono/Projects/xerv/crates/xerv-core/tests/api_contract.rs`:

```rust
//! Testy kontraktu API Punktu 2.
//!
//! Gwarantują, że Rdzeń wystawia typy zadeklarowane w planie w obu
//! lokalizacjach (moduł `api` i korzeń `xerv_core`).

#[allow(dead_code)]
mod korzen {
    use xerv_core::{CoreConfig, CoreState, XervCore};
    pub type RootConfig = CoreConfig;
    pub type RootState = CoreState;
    pub type RootCore = XervCore;
}

#[allow(dead_code)]
mod via_api {
    use xerv_core::api::{CoreConfig, CoreState, XervCore};
    pub type ApiConfig = CoreConfig;
    pub type ApiState = CoreState;
    pub type ApiCore = XervCore;
}

#[test]
fn contract_holds_for_config() {
    fn _takes_both<T>(_: T, _: T) {}
    let dir = tempfile::tempdir().unwrap();
    let cfg1: korzen::RootConfig = CoreConfig::default();
    let cfg2: via_api::ApiConfig = xerv_core::api::CoreConfig::default();
    let _ = dir;
    _takes_both(cfg1, cfg2);
}

#[test]
fn contract_holds_for_state() {
    let s1: korzen::RootState = xerv_core::CoreState::fresh();
    let s2: via_api::ApiState = xerv_core::api::CoreState::fresh();
    assert_eq!(s1, s2);
}

#[test]
fn contract_holds_for_core() {
    // XervCore::new jest w obu lokalizacjach tą samą funkcją (typ-argument).
    fn _is_same<T>(_: fn(xerv_core::CoreConfig, std::path::PathBuf) -> xerv_core::Result<T>) {}
    let _ = xerv_core::XervCore::new as _;
    let _ = xerv_core::api::XervCore::new as _;
}
```

Weryfikacja: `cargo test -p xerv-core --test api_contract` → `test result: ok. 3 passed; 0 failed`.

Commit: `test(core): api contract freeze tests`.

---

### Etap 5: Walidacja końcowa

**Task 5.1** — `cargo build` + `cargo test` + `cargo clippy` + `cargo fmt --check` + `cargo doc`.

```bash
cd ~/Projects/xerv
cargo build -p xerv-core 2>&1 | tail -3
cargo test  -p xerv-core 2>&1 | grep -E "^test result|^running"
cargo clippy -p xerv-core --all-targets -- -D warnings 2>&1 | tail -3
cargo fmt   -p xerv-core --check; echo "fmt exit: $?"
cargo doc   -p xerv-core --no-deps 2>&1 | tail -3
```

Oczekiwane:
- `cargo build` → `Finished 'dev' profile`.
- `cargo test` → `test result: ok. 16 + 3 + 3 + 1 + 3 = 26 passed; 0 failed; 0 ignored; 0 measured` (16 z Punktu 1, plus 9 nowych).
- `cargo clippy -- -D warnings` → `Finished` (zero ostrzeżeń).
- `cargo fmt --check` → exit 0.
- `cargo doc` → `Generated .../target/doc/xerv_core/index.html` + brak warningów o broken linki.

**Kryteria zakończenia Punktu 2 (Definition of Done):**
1. Moduł `xerv_core::api` istnieje (jeśli decyzja 1 = a) lub nie istnieje (jeśli decyzja 1 = b).
2. `XervCore::is_shutdown` istnieje (jeśli decyzja 4 = a) lub nie istnieje (jeśli decyzja 4 = b).
3. `XervCore` ma `Debug` (jeśli decyzja 5 = a) lub nie (jeśli decyzja 5 = b).
4. Publiczne typy Rdzenia są dostępne zarówno w `xerv_core::*` jak i w `xerv_core::api::*` (tożsamość typów).
5. `cargo test` zielony dla wszystkich testów (Punkt 1 + Punkt 2).
6. `cargo clippy -- -D warnings` zielony.
7. `cargo fmt --check` zielony.
8. `cargo doc --no-deps` zielony, bez broken-linków.
9. Brak nowych zależności runtime/dev.
10. Brak plików `bin/`, `examples/`, nowych crate'ów w workspace.
11. Brak traitów, rejestrów, subskrypcji — scope zachowany.
12. Brak breaking changes w stosunku do Punktu 1 (sygnatury istniejących metod nietknięte).

---

## Tests / validation

- **Pełen przebieg** po każdym etapie: `cargo test -p xerv-core` → wszystkie zielone.
- **Walidacja końcowa** (Etap 5.1) — pięć komend powyżej.
- **Czego NIE robimy w Punkcie 2**: benchmarki, coverage, CI, cross-compile, release build, migracje, dokumenty poza doc-komentarzami, przykłady (`examples/`).

---

## Risks, tradeoffs, and open questions

**Ryzyka:**
- **Moduł `api` i korzeń mają ten sam typ** — jeśli ktoś doda `pub use` w `lib.rs` innej kolejności, kompilator nic nie złapie; type identity pilnuje test `api_reexports_match_crate_root_types`.
- **`api_version()` zwraca `Version` przez wartość** (z Punktu 1). Konsumenci, którzy potrzebują `&'static`, nie dostaną tego — jeśli przyszły Punkt tego wymaga, doda się `static` bufor, ale **będzie to** breaking change → bump wersji.
- **`is_shutdown()` wewnętrznie używa `Mutex<bool>`** zamiast `AtomicBool` — wybór spójności z resztą Rdzenia, nie wydajności. W przyszłym Punkcie można zamienić.
- **Oznaczenia `@stable` / `@frozen` w doc-komentarzu** są **niewidoczne dla kompilatora** — to konwencja, nie mechanizm. Jeśli projekt potrzebuje silniejszej gwarancji (np. `#[doc(hidden)]` na niestabilnych, osobny `xerv_core::internal`), to już osobny temat poza Punktem 2.

**Tradeoffi (świadomie wybrane, do potwierdzenia):**
- **Moduł `api` to cienki reexport, nie osobny namespace typów.** Nie tworzymy „drugiej klasy" typów — Rdzeń ma być jeden, `api` jest tylko drzwiami do niego. Alternatywa (osobne typy z `From`) to już feature, nie bug fix.
- **Brak traitów rozszerzeń w Punkcie 2.** Konsumenci dostaną `&XervCore` i same zdecydują, jak go opakować (kompozycja, nie trait). To jest **kompozycja ponad dziedziczenie** w duchu Rust.
- **Brak nowego `Cargo.toml` ani osobnego crate'a.** `api` to `pub mod` wewnątrz `xerv-core` — żadnych nowych `members` w workspace, żadnych dodatkowych zależności.
- **`API_VERSION` zostaje `0.1.0`.** Punkt 2 jest kompatybilny wstecz z Punktem 1. Jeśli użytkownik wskaże inaczej w decyzji 8, bump nastąpi w Etapie 0 przed commitem `api`.

**Otwarte pytania (przeniesione do sekcji „Decyzje wymagające potwierdzenia"):**
- Czy moduł `api` istnieje; aliasy `ApiError`/`ApiResult`; oznaczenia stabilności w doc; `is_shutdown`; `Debug`; podstawa rozszerzalności (wariant a/b/c); `Cargo.lock`; bump wersji API; sygnatura `api_version`; format doc-komentarzy.

**Pytania odrzucone (nie w Punkcie 2):**
- Jak rejestrować pluginy / moduły / adaptery? — **przyszły punkt (TUI/CLI/moduły)**.
- Jak wygląda subskrypcja zdarzeń? — **przyszły punkt**.
- Jak wygląda async API? — **przyszły punkt**.
- Jak wygląda integracja z VPS / xerv.pl / registry? — **przyszły punkt**.
- Jak wygląda interfejs dla użytkownika końcowego? — **przyszły punkt (TUI/CLI)**.

---

## Deliverable tego planu

Ścieżka: `/home/xono/.hermes/plans/2026-09-02_140010-xerv-core-point-2-api.md`
