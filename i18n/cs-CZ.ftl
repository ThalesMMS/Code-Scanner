# Code Scanner — Čeština (Česko)

app-name = Code Scanner
app-banner =
    ╔═══════════════════════════════════════════════════════════════╗
    ║               RUST CODE SCANNER (UNIFIED)                     ║
    ╚═══════════════════════════════════════════════════════════════╝
app-about = Spojí kódovou základnu do jedné textové sestavy pro audity, revize kódu a prompty LLM.

cli-arg-input-help = Vstupní adresář (projekt ke skenování)
cli-arg-output-help = Výstupní adresář pro sestavy
cli-arg-loc-help = Režim LOC: spočítá souhrn řádků a tokenů pro zadanou cestu
cli-arg-no-gitignore-help = Ignorovat soubor .gitignore projektu
cli-arg-verbose-help = Podrobný režim
cli-arg-ignore-help = Přípony souborů k ignorování (např. --ignore .ts .js .json)
cli-arg-max-output-lines-help = Rozdělit výstup do více souborů po tomto počtu řádků (nikdy uprostřed zdrojového souboru)
cli-arg-lang-help = Jazyk zpráv CLI (např. en-US, cs-CZ)

cli-label-input = 📍 Vstup: { $path }
cli-label-output = 📍 Výstup: { $path }
cli-max-output-lines = 📄 Max. počet řádků výstupu na soubor: { $count }
cli-default-dir-created =
    ℹ️  Výchozí vstupní adresář byl vytvořen v: { $path }
    Přidejte sem projekty k analýze a spusťte znovu.
cli-completed = ✨ Hotovo!
cli-no-subdirectories = ℹ️  Nenalezeny žádné podadresáře. Kořen se zpracuje jako jeden projekt.
cli-loc-mode = 📏 Režim LOC
cli-loc-target = 📍 Cíl: { $path }
cli-loc-summary-target = 📁 Cíl: { $path }
cli-gitignore-disabled = ⚠️  .gitignore je vypnutý (--no-gitignore)
cli-loc-summary-title = 📊 SOUHRN LOC
cli-loc-files-processed = ✅ Zpracované soubory: { $count }
cli-loc-files-skipped = ⏭️  Přeskočené soubory: { $count }
cli-loc-total-lines = 🧮 Celkem řádků: { $count }
cli-loc-total-chars = 🔤 Celkem znaků: { $count }
cli-loc-estimated-tokens = 🤖 Odhadované tokeny: { $count }
cli-loc-top-files = 📈 10 SOUBORŮ S NEJVÍCE ŘÁDKY
cli-loc-no-files =   (žádné započítané soubory)
cli-loc-top-file =   { $index }. { $path } ({ $lines } řádků, { $size })
cli-loc-dir-not-found = Adresář pro LOC nebyl nalezen: { $path }
cli-processing = 📦 Zpracování: { $project } ({ $kind })
cli-saved-to =   ✅ Uloženo do: { $path }
cli-ignoring-excessive-size = Ignoruje se { $path } (nadměrná velikost)
cli-error-reading-input = Chyba při čtení vstupu: { $error }
cli-error-input-not-found = Vstupní adresář nebyl nalezen: { $path }
cli-error-create-input = Nepodařilo se vytvořit vstupní adresář: { $path }
cli-error-create-output = Nepodařilo se vytvořit výstupní adresář
cli-error-create-report = Nepodařilo se vytvořit výstupní soubor: { $path }
cli-error-metadata = Nepodařilo se načíst metadata { $path }
cli-report-files-processed =   ✅ Zpracované soubory: { $count }

report-folder-structure = STRUKTURA SLOŽEK
report-file-contents = 📄 OBSAH SOUBORŮ
report-contents-rule = ═══════════════════════════════════════════════════════════════
report-continued-part = (pokračování — část { $part })
report-file-rule-top = ┌─────────────────────────────────────────────────────────────
report-file-name-verbose = │ 📄 { $path }
report-file-size-verbose = │ 📊 Velikost: { $size }
report-file-rule-mid = ├─────────────────────────────────────────────────────────────
report-file-rule-bottom = └─────────────────────────────────────────────────────────────
report-file-name = 📄 { $path }
report-binary-omitted-verbose = │ [Binární soubor nebo nepodporované kódování — obsah vynechán]
report-binary-omitted = [Binární soubor nebo nepodporované kódování — obsah vynechán]
report-utf8-error-verbose = │ [Chyba při čtení souboru jako textu UTF-8]
report-utf8-error = [Chyba při čtení souboru jako textu UTF-8]
report-summary-title = 📊 SOUHRN
report-files-skipped =   ⏭️  Přeskočené soubory (odhad): { $count }
report-total-content-size =   💾 Celková velikost obsahu: { $size }

skip-reason-ignored-file = ignorovaný soubor
skip-reason-hidden-file = skrytý soubor
skip-reason-ignored-extension = ignorovaná přípona
skip-reason-unsupported-extension = nepodporovaná přípona
skip-reason-metadata-error = chyba metadat
skip-reason-over-max-file-size = překročená max. velikost souboru
skip-reason-binary-file = binární soubor
skip-reason-read-error = chyba čtení
skip-reason-walk-error = chyba procházení

tauri-error-input-missing = Vstupní složka neexistuje: { $path }
tauri-error-folder-missing = Složka neexistuje: { $path }
tauri-error-running-app = chyba při spouštění aplikace Code Scanner

ui-toggle-theme = Přepnout motiv
ui-project-folder = Složka projektu
ui-project-folder-placeholder = Vyberte složku projektu
ui-browse = Procházet
ui-recent = Nedávné
ui-no-recent-projects = Žádné nedávné projekty
ui-analyze = Analyzovat
ui-run-scan = Spustit sken
ui-idle = Nečinný
ui-project-controls-aria = Ovládání projektu
ui-active-project = Aktivní projekt
ui-no-project-selected = Není vybrán žádný projekt
ui-choose-folder = Vyberte složku a začněte.
ui-project-type-generic = Obecný
ui-metric-files = Soubory
ui-metric-lines = Řádky
ui-metric-tokens = Tokeny
ui-metric-text-size = Velikost textu
ui-metric-skipped = Přeskočeno
ui-metric-depth = Hloubka
ui-composition = Složení
ui-extension-mix = Mix přípon
ui-groups-count = { $count } skupin
ui-extension-token-mix-aria = Mix tokenů podle přípon
ui-distribution = Rozložení
ui-largest-files = Největší soubory
ui-largest-files-metric-aria = Metrika největších souborů
ui-segment-tokens = Tokeny
ui-segment-lines = Řádky
ui-segment-size = Velikost
ui-llm-readiness = Připravenost pro LLM
ui-awaiting-analysis = Čeká se na analýzu
ui-scanner-diagnostics = Diagnostika skeneru
ui-inclusion-rules = Pravidla zahrnutí
ui-config = Konfigurace
ui-gitignore = Gitignore
ui-max-file-size = Max. velikost souboru
ui-scan-setup = Nastavení skenování
ui-options = Možnosti
ui-standard = Standardní
ui-verbose-review = Podrobná revize
ui-split-safe = Bezpečné rozdělení
ui-ignore-lock-heavy = Ignorovat lock/těžké
ui-output-folder = Výstupní složka
ui-output-placeholder = Složka sestav
ui-choose-output-folder = Vyberte výstupní složku
ui-verbose-report = Podrobná sestava
ui-ignore-gitignore = Ignorovat .gitignore
ui-extensions-to-exclude = Přípony k vyloučení
ui-split-after-n-lines = Rozdělit výstup po N řádcích
ui-no-limit = Bez limitu
ui-local-history = Místní historie
ui-recent-runs = Nedávné běhy
ui-output = Výstup
ui-saved-reports = Uložené sestavy
ui-status-analyzing = Analýza projektu
ui-status-scanning = Probíhá skenování
ui-status-analysis-failed = Analýza selhala
ui-status-scan-failed = Skenování selhalo
ui-status-recent-loaded = Načten nedávný projekt
ui-status-report-copied = Cesta k sestavě zkopírována
ui-unnamed-project = Nepojmenovaný projekt
ui-recent-project = Nedávný projekt
ui-project-config = Konfigurace projektu
ui-defaults = Výchozí
ui-enabled = Zapnuto
ui-disabled = Vypnuto
ui-saved-reports-count = { $count ->
    [one] { $count } uložená sestava
    [few] { $count } uložené sestavy
    [many] { $count } uložených sestav
   *[other] { $count } uložených sestav
}
ui-language = Jazyk
ui-language-system = Systém
ui-analyze-to-refresh = Analyzujte pro obnovení
ui-analyze-to-refresh-distribution = Analyzujte pro obnovení rozložení souborů.
ui-analyze-to-refresh-readiness = Analyzujte pro obnovení připravenosti.
ui-analyze-to-refresh-diagnostics = Analyzujte pro obnovení diagnostiky.
ui-no-source-files = Žádné zdrojové soubory
ui-donut-tokens = tokeny
ui-legend-files = { $percent }% - { $files } souborů
ui-analyze-to-populate = Analyzujte projekt, aby se vyplnilo rozložení souborů.
ui-no-files-matched = Žádné soubory neodpovídají pravidlům skeneru.
ui-readiness-compact = Kompaktní prompt
ui-readiness-review = Prompt k revizi
ui-readiness-large = Velký kontext
ui-insight-tokens-files = { $tokens } odhadovaných tokenů v { $files } souborech.
ui-insight-split-active = Rozdělení výstupu je aktivní: { $lines } řádků na část sestavy.
ui-insight-split-suggest = Nastavte rozdělení výstupu kolem { $lines } řádků, abyste zmenšili příliš velké sestavy.
ui-insight-split-optional = Při aktuálním počtu řádků je rozdělení výstupu volitelné.
ui-insight-top-file = { $path } přispívá nejvíce tokeny: { $tokens }.
ui-insight-top-extension = Soubory { $extension } tvoří { $percent }% odhadovaných tokenů.
ui-insight-gitignore-disabled = .gitignore je vypnutý, takže vygenerované nebo dodané závislosti se mohou dostat do skenu.
ui-no-skipped-files = Žádné přeskočené soubory.
ui-no-saved-reports = Zatím žádné uložené sestavy.
ui-reveal-in-finder = Zobrazit ve Finderu
ui-copy-path = Kopírovat cestu
ui-error-copy-path = Cestu k sestavě se nepodařilo zkopírovat.
ui-choose-recent-project = Vyberte nedávný projekt
ui-no-local-history = Zatím žádná místní historie.
ui-history-tokens = { $tokens } tokenů
ui-not-analyzed = neanalyzováno
ui-error-choose-folder-analyze = Před analýzou vyberte složku projektu.
ui-error-choose-folder-scan = Před skenováním vyberte složku projektu.
ui-status-analyzed-files = Analyzováno { $count } souborů
ui-error-analyze = Tento projekt se nepodařilo analyzovat.
ui-status-saved-reports = { $count ->
    [one] Uložen { $count } soubor sestavy
    [few] Uloženy { $count } soubory sestavy
    [many] Uloženo { $count } souborů sestavy
   *[other] Uloženo { $count } souborů sestavy
}
ui-error-scan = Skenování se nepodařilo spustit.
ui-status-project-selected = Projekt vybrán
ui-status-preset-applied = Předvolba { $preset } použita
ui-analyze-to-calculate-readiness = Analyzujte projekt, abyste spočítali připravenost promptu.
ui-analyze-to-inspect-skipped = Analyzujte projekt, abyste zkontrolovali přeskočené soubory.
ui-no-analysis-yet = Zatím žádná analýza
ui-zero-bytes = 0 B

bash-banner =
    ╔═══════════════════════════════════════════════════════════════╗
    ║                CODE PROJECT SCANNER                           ║
    ╚═══════════════════════════════════════════════════════════════╝
bash-processing =   📁 Zpracování: { $project }
bash-detected-type =     🔍 Zjištěný typ: { $kind }
bash-using-gitignore =     📋 Používá se .gitignore projektu
bash-error-generating-tree = Chyba při generování stromu
bash-files-found =     📊 Nalezené soubory: { $count }
bash-skipped-too-large = ⚠️  PŘESKOČENO: Příliš velké ({ $size } > { $max })
bash-skipped-too-large-verbose = │ ⚠️  PŘESKOČENO: Příliš velké ({ $size } > { $max })
bash-error-reading-file = [Chyba při čtení souboru]
bash-error-reading-file-verbose = │ [Chyba při čtení souboru]
bash-binary-omitted = [Binární soubor — vynechán]
bash-binary-omitted-verbose = │ [Binární soubor — vynechán]
bash-summary-skipped-gitignore =   📋 Přeskočeno přes .gitignore: { $count }
bash-processed =     ✅ Zpracováno: { $count }
bash-skipped =     ⏭️  Přeskočeno: { $count }
bash-via-gitignore =     📋 Přes .gitignore: { $count }
bash-size =     💾 Velikost: { $size }
bash-configuration = 📍 Konfigurace:
bash-target-directory =    • Cílový adresář: { $path }
bash-output-directory =    • Výstupní adresář: { $path }
bash-output-filename-suffix =    • Přípona názvu výstupního souboru: { $suffix }
bash-max-file-size =    • Max. velikost souboru: { $size }
bash-use-gitignore =    • Použít .gitignore: { $value }
bash-verbose-mode =    • Podrobný režim: { $value }
bash-extra-ignored-files =    • Další ignorované soubory: { $value }
bash-extra-ignored-dirs =    • Další ignorované adresáře: { $value }
bash-ignored-relative-paths =    • Ignorované relativní cesty: { $value }
bash-ignored-absolute-paths =    • Ignorované absolutní cesty: { $value }
bash-starting-scan = 🚀 Spouštění skenování...
bash-project-n = [Projekt { $count }]
bash-saved =   💾 Uloženo: { $path }
bash-no-subdirectories = ℹ️  Nenalezeny žádné podadresáře. Zpracovává se { $path } jako jeden projekt...
bash-done = ✨ HOTOVO!
bash-total-projects =   📊 Celkem zpracovaných projektů: { $count }
bash-files-generated-in =   📂 Soubory vygenerovány v: { $path }
bash-generated-files = 📋 Vygenerované soubory:
bash-no-files-generated =   ⚠️  Nebyly vygenerovány žádné soubory.
bash-available-env = 💡 Dostupné proměnné prostředí:
bash-env-target-dir =    • TARGET_DIR - Cílový adresář ke skenování
bash-env-output-dir =    • OUTPUT_DIR - Výstupní adresář
bash-env-output-suffix =    • OUTPUT_FILE_SUFFIX - Přípona názvu výstupního souboru
bash-env-max-size =    • MAX_SIZE_BYTES - Max. velikost souboru
bash-env-use-gitignore =    • USE_GITIGNORE - Použít .gitignore (true/false)
bash-env-verbose =    • VERBOSE - Podrobný režim (true/false)
bash-env-ignore-files =    • IGNORE_FILES_EXTRA - Další soubory k ignorování (oddělené svislou čárou)
bash-env-ignore-dirs =    • IGNORE_DIRS_EXTRA - Další adresáře k ignorování (oddělené svislou čárou)
bash-env-ignore-paths =    • IGNORE_PATHS - Konkrétní relativní cesty k ignorování (oddělené svislou čárou)
bash-env-ignore-absolute =    • IGNORE_ABSOLUTE_PATHS - Konkrétní absolutní cesty k ignorování (oddělené svislou čárou)
bash-quick-examples = 📌 Rychlé příklady:
bash-default-dir-hint =    Přidejte sem projekty ke skenování a spusťte skript znovu.
bash-error-target-not-found = ❌ Chyba: Cílový adresář nebyl nalezen: { $path }
bash-report-total-size =   💾 Celková velikost: { $size }

cli-help-usage = Použití:
cli-help-options = Možnosti:
cli-help-print-help = Vypsat nápovědu
cli-help-print-version = Vypsat verzi
