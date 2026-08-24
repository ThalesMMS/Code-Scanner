# Code Scanner — Polski (Polska)

app-name = Code Scanner
app-banner =
    ╔═══════════════════════════════════════════════════════════════╗
    ║               RUST CODE SCANNER (UNIFIED)                     ║
    ╚═══════════════════════════════════════════════════════════════╝
app-about = Łączy bazę kodu w jeden raport tekstowy do audytów, przeglądów kodu i promptów LLM.

cli-arg-input-help = Katalog wejściowy (projekt do skanowania)
cli-arg-output-help = Katalog wyjściowy raportów
cli-arg-loc-help = Tryb LOC: podsumowanie linii i tokenów dla wskazanej ścieżki
cli-arg-no-gitignore-help = Ignoruj plik .gitignore projektu
cli-arg-verbose-help = Tryb szczegółowy
cli-arg-ignore-help = Rozszerzenia plików do zignorowania (np. --ignore .ts .js .json)
cli-arg-max-output-lines-help = Podziel wynik na kilka plików po tej liczbie linii (nigdy w środku pliku źródłowego)
cli-arg-lang-help = Język komunikatów CLI (np. en-US, pl-PL)

cli-label-input = 📍 Wejście: { $path }
cli-label-output = 📍 Wyjście: { $path }
cli-max-output-lines = 📄 Maks. liczba linii wyniku na plik: { $count }
cli-default-dir-created =
    ℹ️  Domyślny katalog wejściowy został utworzony w: { $path }
    Dodaj projekty do analizy w tym katalogu i uruchom ponownie.
cli-completed = ✨ Zakończono!
cli-no-subdirectories = ℹ️  Nie znaleziono podkatalogów. Przetwarzanie katalogu głównego jako jednego projektu.
cli-loc-mode = 📏 Tryb LOC
cli-loc-target = 📍 Cel: { $path }
cli-loc-summary-target = 📁 Cel: { $path }
cli-gitignore-disabled = ⚠️  .gitignore wyłączony (--no-gitignore)
cli-loc-summary-title = 📊 PODSUMOWANIE LOC
cli-loc-files-processed = ✅ Przetworzone pliki: { $count }
cli-loc-files-skipped = ⏭️  Pominięte pliki: { $count }
cli-loc-total-lines = 🧮 Łączna liczba linii: { $count }
cli-loc-total-chars = 🔤 Łączna liczba znaków: { $count }
cli-loc-estimated-tokens = 🤖 Szacowane tokeny: { $count }
cli-loc-top-files = 📈 10 PLIKÓW Z NAJWIĘKSZĄ LICZBĄ LINII
cli-loc-no-files =   (brak zliczonych plików)
cli-loc-top-file =   { $index }. { $path } ({ $lines } linii, { $size })
cli-loc-dir-not-found = Nie znaleziono katalogu dla LOC: { $path }
cli-processing = 📦 Przetwarzanie: { $project } ({ $kind })
cli-saved-to =   ✅ Zapisano w: { $path }
cli-ignoring-excessive-size = Ignorowanie { $path } (zbyt duży rozmiar)
cli-error-reading-input = Błąd odczytu danych wejściowych: { $error }
cli-error-input-not-found = Nie znaleziono katalogu wejściowego: { $path }
cli-error-create-input = Nie udało się utworzyć katalogu wejściowego: { $path }
cli-error-create-output = Nie udało się utworzyć katalogu wyjściowego
cli-error-create-report = Nie udało się utworzyć pliku wyjściowego: { $path }
cli-error-metadata = Nie udało się odczytać metadanych { $path }
cli-report-files-processed =   ✅ Przetworzone pliki: { $count }

report-folder-structure = STRUKTURA FOLDERÓW
report-file-contents = 📄 ZAWARTOŚĆ PLIKÓW
report-contents-rule = ═══════════════════════════════════════════════════════════════
report-continued-part = (kontynuacja — część { $part })
report-file-rule-top = ┌─────────────────────────────────────────────────────────────
report-file-name-verbose = │ 📄 { $path }
report-file-size-verbose = │ 📊 Rozmiar: { $size }
report-file-rule-mid = ├─────────────────────────────────────────────────────────────
report-file-rule-bottom = └─────────────────────────────────────────────────────────────
report-file-name = 📄 { $path }
report-binary-omitted-verbose = │ [Plik binarny lub nieobsługiwane kodowanie — pominięto zawartość]
report-binary-omitted = [Plik binarny lub nieobsługiwane kodowanie — pominięto zawartość]
report-utf8-error-verbose = │ [Błąd odczytu pliku jako tekstu UTF-8]
report-utf8-error = [Błąd odczytu pliku jako tekstu UTF-8]
report-summary-title = 📊 PODSUMOWANIE
report-files-skipped =   ⏭️  Pominięte pliki (szacunkowo): { $count }
report-total-content-size =   💾 Łączny rozmiar zawartości: { $size }

skip-reason-ignored-file = zignorowany plik
skip-reason-hidden-file = plik ukryty
skip-reason-ignored-extension = zignorowane rozszerzenie
skip-reason-unsupported-extension = nieobsługiwane rozszerzenie
skip-reason-metadata-error = błąd metadanych
skip-reason-over-max-file-size = przekroczony maks. rozmiar pliku
skip-reason-binary-file = plik binarny
skip-reason-read-error = błąd odczytu
skip-reason-walk-error = błąd przeglądania

tauri-error-input-missing = Folder wejściowy nie istnieje: { $path }
tauri-error-folder-missing = Folder nie istnieje: { $path }
tauri-error-running-app = błąd podczas uruchamiania aplikacji Code Scanner

ui-toggle-theme = Przełącz motyw
ui-project-folder = Folder projektu
ui-project-folder-placeholder = Wybierz folder projektu
ui-browse = Przeglądaj
ui-recent = Ostatnie
ui-no-recent-projects = Brak ostatnich projektów
ui-analyze = Analizuj
ui-run-scan = Uruchom skan
ui-idle = Bezczynny
ui-project-controls-aria = Sterowanie projektem
ui-active-project = Aktywny projekt
ui-no-project-selected = Nie wybrano projektu
ui-choose-folder = Wybierz folder, aby zacząć.
ui-project-type-generic = Ogólny
ui-metric-files = Pliki
ui-metric-lines = Linie
ui-metric-tokens = Tokeny
ui-metric-text-size = Rozmiar tekstu
ui-metric-skipped = Pominięte
ui-metric-depth = Głębokość
ui-composition = Skład
ui-extension-mix = Mix rozszerzeń
ui-groups-count = { $count } grup
ui-extension-token-mix-aria = Mix tokenów według rozszerzeń
ui-distribution = Rozkład
ui-largest-files = Największe pliki
ui-largest-files-metric-aria = Metryka największych plików
ui-segment-tokens = Tokeny
ui-segment-lines = Linie
ui-segment-size = Rozmiar
ui-llm-readiness = Gotowość na LLM
ui-awaiting-analysis = Oczekiwanie na analizę
ui-scanner-diagnostics = Diagnostyka skanera
ui-inclusion-rules = Reguły uwzględniania
ui-config = Konfiguracja
ui-gitignore = Gitignore
ui-max-file-size = Maks. rozmiar pliku
ui-scan-setup = Konfiguracja skanowania
ui-options = Opcje
ui-standard = Standardowy
ui-verbose-review = Szczegółowy przegląd
ui-split-safe = Bezpieczny podział
ui-ignore-lock-heavy = Ignoruj lock/ciężkie
ui-output-folder = Folder wyjściowy
ui-output-placeholder = Folder raportów
ui-choose-output-folder = Wybierz folder wyjściowy
ui-verbose-report = Szczegółowy raport
ui-ignore-gitignore = Ignoruj .gitignore
ui-extensions-to-exclude = Rozszerzenia do wykluczenia
ui-split-after-n-lines = Podziel wynik po N liniach
ui-no-limit = Bez limitu
ui-local-history = Historia lokalna
ui-recent-runs = Ostatnie uruchomienia
ui-output = Wynik
ui-saved-reports = Zapisane raporty
ui-status-analyzing = Analizowanie projektu
ui-status-scanning = Skanowanie
ui-status-analysis-failed = Analiza nie powiodła się
ui-status-scan-failed = Skanowanie nie powiodło się
ui-status-recent-loaded = Wczytano ostatni projekt
ui-status-report-copied = Skopiowano ścieżkę raportu
ui-unnamed-project = Projekt bez nazwy
ui-recent-project = Ostatni projekt
ui-project-config = Konfiguracja projektu
ui-defaults = Domyślne
ui-enabled = Włączone
ui-disabled = Wyłączone
ui-saved-reports-count = { $count ->
    [one] { $count } zapisany raport
    [few] { $count } zapisane raporty
    [many] { $count } zapisanych raportów
   *[other] { $count } zapisanych raportów
}
ui-language = Język
ui-language-system = System
ui-analyze-to-refresh = Analizuj, aby odświeżyć
ui-analyze-to-refresh-distribution = Analizuj, aby odświeżyć rozkład plików.
ui-analyze-to-refresh-readiness = Analizuj, aby odświeżyć gotowość.
ui-analyze-to-refresh-diagnostics = Analizuj, aby odświeżyć diagnostykę.
ui-no-source-files = Brak plików źródłowych
ui-donut-tokens = tokeny
ui-legend-files = { $percent }% - { $files } plików
ui-analyze-to-populate = Analizuj projekt, aby wypełnić rozkład plików.
ui-no-files-matched = Żadne pliki nie pasują do reguł skanera.
ui-readiness-compact = Zwięzły prompt
ui-readiness-review = Prompt do przeglądu
ui-readiness-large = Duży kontekst
ui-insight-tokens-files = { $tokens } szacowanych tokenów w { $files } plikach.
ui-insight-split-active = Podział wyniku jest aktywny: { $lines } linii na część raportu.
ui-insight-split-suggest = Ustaw podział wyniku około { $lines } linii, aby zmniejszyć zbyt duże raporty.
ui-insight-split-optional = Przy bieżącej liczbie linii podział wyniku jest opcjonalny.
ui-insight-top-file = { $path } wnosi najwięcej tokenów: { $tokens }.
ui-insight-top-extension = Pliki { $extension } stanowią { $percent }% szacowanych tokenów.
ui-insight-gitignore-disabled = .gitignore jest wyłączony, więc wygenerowane lub dołączone biblioteki mogą wejść do skanu.
ui-no-skipped-files = Brak pominiętych plików.
ui-no-saved-reports = Brak zapisanych raportów.
ui-reveal-in-finder = Pokaż w Finderze
ui-copy-path = Kopiuj ścieżkę
ui-error-copy-path = Nie udało się skopiować ścieżki raportu.
ui-choose-recent-project = Wybierz ostatni projekt
ui-no-local-history = Brak lokalnej historii.
ui-history-tokens = { $tokens } tokenów
ui-not-analyzed = nieanalizowany
ui-error-choose-folder-analyze = Przed analizą wybierz folder projektu.
ui-error-choose-folder-scan = Przed skanowaniem wybierz folder projektu.
ui-status-analyzed-files = Przeanalizowano { $count } plików
ui-error-analyze = Nie udało się przeanalizować tego projektu.
ui-status-saved-reports = { $count ->
    [one] Zapisano { $count } plik raportu
    [few] Zapisano { $count } pliki raportu
    [many] Zapisano { $count } plików raportu
   *[other] Zapisano { $count } plików raportu
}
ui-error-scan = Nie udało się uruchomić skanowania.
ui-status-project-selected = Wybrano projekt
ui-status-preset-applied = Zastosowano preset { $preset }
ui-analyze-to-calculate-readiness = Analizuj projekt, aby obliczyć gotowość promptu.
ui-analyze-to-inspect-skipped = Analizuj projekt, aby sprawdzić pominięte pliki.
ui-no-analysis-yet = Brak analizy
ui-zero-bytes = 0 B

bash-banner =
    ╔═══════════════════════════════════════════════════════════════╗
    ║                CODE PROJECT SCANNER                           ║
    ╚═══════════════════════════════════════════════════════════════╝
bash-processing =   📁 Przetwarzanie: { $project }
bash-detected-type =     🔍 Wykryty typ: { $kind }
bash-using-gitignore =     📋 Używam .gitignore projektu
bash-error-generating-tree = Błąd generowania drzewa
bash-files-found =     📊 Znalezione pliki: { $count }
bash-skipped-too-large = ⚠️  POMINIĘTO: Zbyt duży ({ $size } > { $max })
bash-skipped-too-large-verbose = │ ⚠️  POMINIĘTO: Zbyt duży ({ $size } > { $max })
bash-error-reading-file = [Błąd odczytu pliku]
bash-error-reading-file-verbose = │ [Błąd odczytu pliku]
bash-binary-omitted = [Plik binarny — pominięto]
bash-binary-omitted-verbose = │ [Plik binarny — pominięto]
bash-summary-skipped-gitignore =   📋 Pominięte przez .gitignore: { $count }
bash-processed =     ✅ Przetworzono: { $count }
bash-skipped =     ⏭️  Pominięto: { $count }
bash-via-gitignore =     📋 Przez .gitignore: { $count }
bash-size =     💾 Rozmiar: { $size }
bash-configuration = 📍 Konfiguracja:
bash-target-directory =    • Katalog docelowy: { $path }
bash-output-directory =    • Katalog wyjściowy: { $path }
bash-output-filename-suffix =    • Sufiks nazwy pliku wyjściowego: { $suffix }
bash-max-file-size =    • Maks. rozmiar pliku: { $size }
bash-use-gitignore =    • Użyj .gitignore: { $value }
bash-verbose-mode =    • Tryb szczegółowy: { $value }
bash-extra-ignored-files =    • Dodatkowe ignorowane pliki: { $value }
bash-extra-ignored-dirs =    • Dodatkowe ignorowane katalogi: { $value }
bash-ignored-relative-paths =    • Ignorowane ścieżki względne: { $value }
bash-ignored-absolute-paths =    • Ignorowane ścieżki bezwzględne: { $value }
bash-starting-scan = 🚀 Rozpoczynanie skanowania...
bash-project-n = [Projekt { $count }]
bash-saved =   💾 Zapisano: { $path }
bash-no-subdirectories = ℹ️  Nie znaleziono podkatalogów. Przetwarzanie { $path } jako jednego projektu...
bash-done = ✨ GOTOWE!
bash-total-projects =   📊 Łączna liczba przetworzonych projektów: { $count }
bash-files-generated-in =   📂 Pliki wygenerowane w: { $path }
bash-generated-files = 📋 Wygenerowane pliki:
bash-no-files-generated =   ⚠️  Nie wygenerowano plików.
bash-available-env = 💡 Dostępne zmienne środowiskowe:
bash-env-target-dir =    • TARGET_DIR - Katalog docelowy do skanowania
bash-env-output-dir =    • OUTPUT_DIR - Katalog wyjściowy
bash-env-output-suffix =    • OUTPUT_FILE_SUFFIX - Sufiks nazwy pliku wyjściowego
bash-env-max-size =    • MAX_SIZE_BYTES - Maks. rozmiar pliku
bash-env-use-gitignore =    • USE_GITIGNORE - Użyj .gitignore (true/false)
bash-env-verbose =    • VERBOSE - Tryb szczegółowy (true/false)
bash-env-ignore-files =    • IGNORE_FILES_EXTRA - Dodatkowe pliki do zignorowania (oddzielone pionową kreską)
bash-env-ignore-dirs =    • IGNORE_DIRS_EXTRA - Dodatkowe katalogi do zignorowania (oddzielone pionową kreską)
bash-env-ignore-paths =    • IGNORE_PATHS - Określone ścieżki względne do zignorowania (oddzielone pionową kreską)
bash-env-ignore-absolute =    • IGNORE_ABSOLUTE_PATHS - Określone ścieżki bezwzględne do zignorowania (oddzielone pionową kreską)
bash-quick-examples = 📌 Szybkie przykłady:
bash-default-dir-hint =    Dodaj projekty do skanowania w tym katalogu i uruchom skrypt ponownie.
bash-error-target-not-found = ❌ Błąd: Nie znaleziono katalogu docelowego: { $path }
bash-report-total-size =   💾 Łączny rozmiar: { $size }

cli-help-usage = Użycie:
cli-help-options = Opcje:
cli-help-print-help = Wyświetl pomoc
cli-help-print-version = Wyświetl wersję
