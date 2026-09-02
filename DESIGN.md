---
version: alpha
name: Xerv TUI
description: Minimalistyczny terminalowy system manager — dark aesthetic, hierarchia przez kontrast i ograniczone akcenty (cyan = interakcja, magenta = wybrane informacje).
colors:
  primary: "#00D7D7"
  secondary: "#D787D7"
  neutral: "#FFFFFF"
  muted: "#585858"
  success: "#5FD75F"
  danger: "#FF5F5F"
typography:
  label:
    fontFamily: terminal monospace
    fontSize: 16px
    fontWeight: 400
  value:
    fontFamily: terminal monospace
    fontSize: 16px
    fontWeight: 400
  emphasis:
    fontFamily: terminal monospace
    fontSize: 16px
    fontWeight: 700
components:
  header-brand:
    textColor: "{colors.primary}"
  header-value:
    textColor: "{colors.secondary}"
  header-meta-label:
    textColor: "{colors.muted}"
  header-meta-value:
    textColor: "{colors.primary}"
  status-ready:
    textColor: "{colors.success}"
  status-shutdown:
    textColor: "{colors.danger}"
  strip-separator:
    textColor: "{colors.muted}"
  strip-value:
    textColor: "{colors.neutral}"
  card-title:
    textColor: "{colors.muted}"
  card-label:
    textColor: "{colors.muted}"
  card-value:
    textColor: "{colors.neutral}"
  card-value-accent:
    textColor: "{colors.secondary}"
  card-separator:
    textColor: "{colors.muted}"
  card-border-idle:
    textColor: "{colors.muted}"
  command-idle:
    textColor: "{colors.muted}"
  command-active-border:
    textColor: "{colors.primary}"
  command-active-text:
    textColor: "{colors.primary}"
  footer-hint:
    textColor: "{colors.primary}"
---

## Overview

Xerv TUI ma wyglądać jak narzędzie systemowe klasy lazygit/rainfrog: ciche,
gęste informacyjnie, z hierarchią budowaną kontrastem i wagą typograficzną,
nie kolorowymi blokami. Tło zawsze pochodzi z motywu terminala użytkownika
(TUI nigdy nie rysuje tła).

## Colors

- **Primary (#00D7D7, terminal `Cyan`):** główny akcent **interaktywny** —
  aktywny slot command bara, brand w headerze, klawisze w footerze, aktywna
  ramka panelu. Cyan nigdy nie jest używany dla statycznych wartości danych.
- **Secondary (#D787D7, terminal `Magenta`):** drugi akcent, zarezerwowany dla
  **wybranych informacji** w headerze (wersja API) i aktywnych wartości w
  kartach. Nigdy dla ramki interaktywnego elementu.
- **Neutral (#FFFFFF, terminal `White`):** wartości danych w kartach i stripie.
  Jaśniejszy niż domyślny fg — wyraźnie odróżnia wartości od etykiet.
- **Muted (#585858, terminal `DarkGray`):** etykiety, separatory, spoczynkowe
  ramki i sloty. Drugi plan wizualny.
- **Success (#5FD75F, `Green`) / Danger (#FF5F5F, `Red`):** wyłącznie status
  READY/SHUTDOWN. Semantyka, nie dekoracja.

## Typography

Terminal monospace, jeden rozmiar. Hierarchia wagą: `BOLD` tylko dla statusu,
brandu i aktywnego slotu. Wyrównanie etykiet do stałej szerokości kolumny.

## Layout

Sekcje pionowo: header (4) → main (min) → footer (1). W main: sidebar (25%) +
dashboard; dashboard = status strip (1) + karty + command bar (3).

## Components

- `card-value-accent` stosujemy tylko dla wartości, które są „odpowiedzią"
  na pytanie użytkownika (wersja API, schema) — reszta wartości neutralna.
- Aktywny command bar: ramka cyan + tekst cyan bold (bez inwersji).
- Spoczynkowe sloty i ramki: muted — nigdy neutral.

## Do's and Don'ts

- **Nie** stosuj cyan/magenta do statycznych etykiet.
- **Nie** inwertuj całych paneli (REVERSED) — maksimum podkreślenia to BOLD.
- **Nie** używaj zielonego/czerwonego poza semantyką statusu.
- **Zawsze** zamykaj ramki slotów — clipping to bug, nie styl.
