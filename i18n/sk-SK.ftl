# Code Scanner — Slovenčina (Slovensko)

app-name = Code Scanner
app-banner =
    ╔═══════════════════════════════════════════════════════════════╗
    ║               RUST CODE SCANNER (UNIFIED)                     ║
    ╚═══════════════════════════════════════════════════════════════╝
app-about = Spojí kódovú základňu do jednej textovej zostavy na audity, kontroly kódu a prompty LLM.

cli-arg-input-help = Vstupný adresár (projekt na skenovanie)
cli-arg-output-help = Výstupný adresár pre zostavy
cli-arg-loc-help = Režim LOC: vypočíta súhrn riadkov a tokenov pre zadanú cestu
cli-arg-no-gitignore-help = Ignorovať súbor .gitignore projektu
cli-arg-verbose-help = Podrobný režim
cli-arg-ignore-help = Prípony súborov na ignorovanie (napr. --ignore .ts .js .json)
cli-arg-max-output-lines-help = Rozdeliť výstup do viacerých súborov po tomto počte riadkov (nikdy uprostred zdrojového súboru)
cli-arg-lang-help = Jazyk správ CLI (napr. en-US, sk-SK)

cli-label-input = 📍 Vstup: { $path }
cli-label-output = 📍 Výstup: { $path }
cli-max-output-lines = 📄 Max. počet riadkov výstupu na súbor: { $count }
cli-default-dir-created =
    ℹ️  Predvolený vstupný adresár bol vytvorený v: { $path }
    Pridajte sem projekty na analýzu a spustite znova.
cli-completed = ✨ Dokončené!
cli-no-subdirectories = ℹ️  Nenašli sa žiadne podadresáre. Koreň sa spracuje ako jeden projekt.
cli-loc-mode = 📏 Režim LOC
cli-loc-target = 📍 Cieľ: { $path }
cli-loc-summary-target = 📁 Cieľ: { $path }
cli-gitignore-disabled = ⚠️  .gitignore je vypnutý (--no-gitignore)
cli-loc-summary-title = 📊 SÚHRN LOC
cli-loc-files-processed = ✅ Spracované súbory: { $count }
cli-loc-files-skipped = ⏭️  Preskočené súbory: { $count }
cli-loc-total-lines = 🧮 Celkom riadkov: { $count }
cli-loc-total-chars = 🔤 Celkom znakov: { $count }
cli-loc-estimated-tokens = 🤖 Odhadované tokeny: { $count }
cli-loc-top-files = 📈 10 SÚBOROV S NAJVÄČŠÍM POČTOM RIADKOV
cli-loc-no-files =   (žiadne započítané súbory)
cli-loc-top-file =   { $index }. { $path } ({ $lines } riadkov, { $size })
cli-loc-dir-not-found = Adresár pre LOC sa nenašiel: { $path }
cli-processing = 📦 Spracovanie: { $project } ({ $kind })
cli-saved-to =   ✅ Uložené do: { $path }
cli-ignoring-excessive-size = Ignoruje sa { $path } (nadmerná veľkosť)
cli-error-reading-input = Chyba pri čítaní vstupu: { $error }
cli-error-input-not-found = Vstupný adresár sa nenašiel: { $path }
cli-error-create-input = Nepodarilo sa vytvoriť vstupný adresár: { $path }
cli-error-create-output = Nepodarilo sa vytvoriť výstupný adresár
cli-error-create-report = Nepodarilo sa vytvoriť výstupný súbor: { $path }
cli-error-metadata = Nepodarilo sa načítať metadáta { $path }
cli-report-files-processed =   ✅ Spracované súbory: { $count }

report-folder-structure = ŠTRUKTÚRA PRIEČINKOV
report-file-contents = 📄 OBSAH SÚBOROV
report-contents-rule = ═══════════════════════════════════════════════════════════════
report-continued-part = (pokračovanie — časť { $part })
report-file-rule-top = ┌─────────────────────────────────────────────────────────────
report-file-name-verbose = │ 📄 { $path }
report-file-size-verbose = │ 📊 Veľkosť: { $size }
report-file-rule-mid = ├─────────────────────────────────────────────────────────────
report-file-rule-bottom = └─────────────────────────────────────────────────────────────
report-file-name = 📄 { $path }
report-binary-omitted-verbose = │ [Binárny súbor alebo nepodporované kódovanie — obsah vynechaný]
report-binary-omitted = [Binárny súbor alebo nepodporované kódovanie — obsah vynechaný]
report-utf8-error-verbose = │ [Chyba pri čítaní súboru ako textu UTF-8]
report-utf8-error = [Chyba pri čítaní súboru ako textu UTF-8]
report-summary-title = 📊 SÚHRN
report-files-skipped =   ⏭️  Preskočené súbory (odhad): { $count }
report-total-content-size =   💾 Celková veľkosť obsahu: { $size }

skip-reason-ignored-file = ignorovaný súbor
skip-reason-hidden-file = skrytý súbor
skip-reason-ignored-extension = ignorovaná prípona
skip-reason-unsupported-extension = nepodporovaná prípona
skip-reason-metadata-error = chyba metadát
skip-reason-over-max-file-size = prekročená max. veľkosť súboru
skip-reason-binary-file = binárny súbor
skip-reason-read-error = chyba čítania
skip-reason-walk-error = chyba prechádzania

tauri-error-input-missing = Vstupný priečinok neexistuje: { $path }
tauri-error-folder-missing = Priečinok neexistuje: { $path }
tauri-error-running-app = chyba pri spúšťaní aplikácie Code Scanner

ui-toggle-theme = Prepnúť motív
ui-project-folder = Priečinok projektu
ui-project-folder-placeholder = Vyberte priečinok projektu
ui-browse = Prehliadať
ui-recent = Nedávne
ui-no-recent-projects = Žiadne nedávne projekty
ui-analyze = Analyzovať
ui-run-scan = Spustiť sken
ui-idle = Nečinný
ui-project-controls-aria = Ovládanie projektu
ui-active-project = Aktívny projekt
ui-no-project-selected = Nie je vybratý žiadny projekt
ui-choose-folder = Vyberte priečinok a začnite.
ui-project-type-generic = Všeobecný
ui-metric-files = Súbory
ui-metric-lines = Riadky
ui-metric-tokens = Tokeny
ui-metric-text-size = Veľkosť textu
ui-metric-skipped = Preskočené
ui-metric-depth = Hĺbka
ui-composition = Zloženie
ui-extension-mix = Mix prípon
ui-groups-count = { $count } skupín
ui-extension-token-mix-aria = Mix tokenov podľa prípon
ui-distribution = Rozloženie
ui-largest-files = Najväčšie súbory
ui-largest-files-metric-aria = Metrika najväčších súborov
ui-segment-tokens = Tokeny
ui-segment-lines = Riadky
ui-segment-size = Veľkosť
ui-llm-readiness = Pripravenosť pre LLM
ui-awaiting-analysis = Čaká sa na analýzu
ui-scanner-diagnostics = Diagnostika skenera
ui-inclusion-rules = Pravidlá zahrnutia
ui-config = Konfigurácia
ui-gitignore = Gitignore
ui-max-file-size = Max. veľkosť súboru
ui-scan-setup = Nastavenie skenovania
ui-options = Možnosti
ui-standard = Štandardný
ui-verbose-review = Podrobná kontrola
ui-split-safe = Bezpečné rozdelenie
ui-ignore-lock-heavy = Ignorovať lock/ťažké
ui-output-folder = Výstupný priečinok
ui-output-placeholder = Priečinok zostáv
ui-choose-output-folder = Vyberte výstupný priečinok
ui-verbose-report = Podrobná zostava
ui-ignore-gitignore = Ignorovať .gitignore
ui-extensions-to-exclude = Prípony na vylúčenie
ui-split-after-n-lines = Rozdeliť výstup po N riadkoch
ui-no-limit = Bez limitu
ui-local-history = Miestna história
ui-recent-runs = Nedávne spustenia
ui-output = Výstup
ui-saved-reports = Uložené zostavy
ui-status-analyzing = Analýza projektu
ui-status-scanning = Prebieha skenovanie
ui-status-analysis-failed = Analýza zlyhala
ui-status-scan-failed = Skenovanie zlyhalo
ui-status-recent-loaded = Načítaný nedávny projekt
ui-status-report-copied = Cesta k zostave skopírovaná
ui-unnamed-project = Nepomenovaný projekt
ui-recent-project = Nedávny projekt
ui-project-config = Konfigurácia projektu
ui-defaults = Predvolené
ui-enabled = Zapnuté
ui-disabled = Vypnuté
ui-saved-reports-count = { $count ->
    [one] { $count } uložená zostava
    [few] { $count } uložené zostavy
    [many] { $count } uložených zostáv
   *[other] { $count } uložených zostáv
}
ui-language = Jazyk
ui-language-system = Systém
ui-analyze-to-refresh = Analyzujte na obnovenie
ui-analyze-to-refresh-distribution = Analyzujte na obnovenie rozloženia súborov.
ui-analyze-to-refresh-readiness = Analyzujte na obnovenie pripravenosti.
ui-analyze-to-refresh-diagnostics = Analyzujte na obnovenie diagnostiky.
ui-no-source-files = Žiadne zdrojové súbory
ui-donut-tokens = tokeny
ui-legend-files = { $percent }% - { $files } súborov
ui-analyze-to-populate = Analyzujte projekt, aby sa vyplnilo rozloženie súborov.
ui-no-files-matched = Žiadne súbory nevyhovujú pravidlám skenera.
ui-readiness-compact = Kompaktný prompt
ui-readiness-review = Prompt na kontrolu
ui-readiness-large = Veľký kontext
ui-insight-tokens-files = { $tokens } odhadovaných tokenov v { $files } súboroch.
ui-insight-split-active = Rozdelenie výstupu je aktívne: { $lines } riadkov na časť zostavy.
ui-insight-split-suggest = Nastavte rozdelenie výstupu okolo { $lines } riadkov, aby ste zmenšili príliš veľké zostavy.
ui-insight-split-optional = Pri aktuálnom počte riadkov je rozdelenie výstupu voliteľné.
ui-insight-top-file = { $path } prispieva najviac tokenmi: { $tokens }.
ui-insight-top-extension = Súbory { $extension } tvoria { $percent }% odhadovaných tokenov.
ui-insight-gitignore-disabled = .gitignore je vypnutý, takže vygenerované alebo dodané závislosti sa môžu dostať do skenu.
ui-no-skipped-files = Žiadne preskočené súbory.
ui-no-saved-reports = Zatiaľ žiadne uložené zostavy.
ui-reveal-in-finder = Zobraziť vo Finderi
ui-copy-path = Kopírovať cestu
ui-error-copy-path = Cestu k zostave sa nepodarilo skopírovať.
ui-choose-recent-project = Vyberte nedávny projekt
ui-no-local-history = Zatiaľ žiadna miestna história.
ui-history-tokens = { $tokens } tokenov
ui-not-analyzed = neanalyzované
ui-error-choose-folder-analyze = Pred analýzou vyberte priečinok projektu.
ui-error-choose-folder-scan = Pred skenovaním vyberte priečinok projektu.
ui-status-analyzed-files = Analyzovaných { $count } súborov
ui-error-analyze = Tento projekt sa nepodarilo analyzovať.
ui-status-saved-reports = { $count ->
    [one] Uložený { $count } súbor zostavy
    [few] Uložené { $count } súbory zostavy
    [many] Uložených { $count } súborov zostavy
   *[other] Uložených { $count } súborov zostavy
}
ui-error-scan = Skenovanie sa nepodarilo spustiť.
ui-status-project-selected = Projekt vybratý
ui-status-preset-applied = Predvoľba { $preset } použitá
ui-analyze-to-calculate-readiness = Analyzujte projekt, aby ste vypočítali pripravenosť promptu.
ui-analyze-to-inspect-skipped = Analyzujte projekt, aby ste skontrolovali preskočené súbory.
ui-no-analysis-yet = Zatiaľ žiadna analýza
ui-zero-bytes = 0 B

bash-banner =
    ╔═══════════════════════════════════════════════════════════════╗
    ║                CODE PROJECT SCANNER                           ║
    ╚═══════════════════════════════════════════════════════════════╝
bash-processing =   📁 Spracovanie: { $project }
bash-detected-type =     🔍 Zistený typ: { $kind }
bash-using-gitignore =     📋 Používa sa .gitignore projektu
bash-error-generating-tree = Chyba pri generovaní stromu
bash-files-found =     📊 Nájdené súbory: { $count }
bash-skipped-too-large = ⚠️  PRESKOČENÉ: Príliš veľké ({ $size } > { $max })
bash-skipped-too-large-verbose = │ ⚠️  PRESKOČENÉ: Príliš veľké ({ $size } > { $max })
bash-error-reading-file = [Chyba pri čítaní súboru]
bash-error-reading-file-verbose = │ [Chyba pri čítaní súboru]
bash-binary-omitted = [Binárny súbor — vynechaný]
bash-binary-omitted-verbose = │ [Binárny súbor — vynechaný]
bash-summary-skipped-gitignore =   📋 Preskočené cez .gitignore: { $count }
bash-processed =     ✅ Spracované: { $count }
bash-skipped =     ⏭️  Preskočené: { $count }
bash-via-gitignore =     📋 Cez .gitignore: { $count }
bash-size =     💾 Veľkosť: { $size }
bash-configuration = 📍 Konfigurácia:
bash-target-directory =    • Cieľový adresár: { $path }
bash-output-directory =    • Výstupný adresár: { $path }
bash-output-filename-suffix =    • Prípona názvu výstupného súboru: { $suffix }
bash-max-file-size =    • Max. veľkosť súboru: { $size }
bash-use-gitignore =    • Použiť .gitignore: { $value }
bash-verbose-mode =    • Podrobný režim: { $value }
bash-extra-ignored-files =    • Ďalšie ignorované súbory: { $value }
bash-extra-ignored-dirs =    • Ďalšie ignorované adresáre: { $value }
bash-ignored-relative-paths =    • Ignorované relatívne cesty: { $value }
bash-ignored-absolute-paths =    • Ignorované absolútne cesty: { $value }
bash-starting-scan = 🚀 Spúšťanie skenovania...
bash-project-n = [Projekt { $count }]
bash-saved =   💾 Uložené: { $path }
bash-no-subdirectories = ℹ️  Nenašli sa žiadne podadresáre. Spracúva sa { $path } ako jeden projekt...
bash-done = ✨ HOTOVO!
bash-total-projects =   📊 Celkom spracovaných projektov: { $count }
bash-files-generated-in =   📂 Súbory vygenerované v: { $path }
bash-generated-files = 📋 Vygenerované súbory:
bash-no-files-generated =   ⚠️  Neboli vygenerované žiadne súbory.
bash-available-env = 💡 Dostupné premenné prostredia:
bash-env-target-dir =    • TARGET_DIR - Cieľový adresár na skenovanie
bash-env-output-dir =    • OUTPUT_DIR - Výstupný adresár
bash-env-output-suffix =    • OUTPUT_FILE_SUFFIX - Prípona názvu výstupného súboru
bash-env-max-size =    • MAX_SIZE_BYTES - Max. veľkosť súboru
bash-env-use-gitignore =    • USE_GITIGNORE - Použiť .gitignore (true/false)
bash-env-verbose =    • VERBOSE - Podrobný režim (true/false)
bash-env-ignore-files =    • IGNORE_FILES_EXTRA - Ďalšie súbory na ignorovanie (oddelené zvislou čiarou)
bash-env-ignore-dirs =    • IGNORE_DIRS_EXTRA - Ďalšie adresáre na ignorovanie (oddelené zvislou čiarou)
bash-env-ignore-paths =    • IGNORE_PATHS - Konkrétne relatívne cesty na ignorovanie (oddelené zvislou čiarou)
bash-env-ignore-absolute =    • IGNORE_ABSOLUTE_PATHS - Konkrétne absolútne cesty na ignorovanie (oddelené zvislou čiarou)
bash-quick-examples = 📌 Rýchle príklady:
bash-default-dir-hint =    Pridajte sem projekty na skenovanie a spustite skript znova.
bash-error-target-not-found = ❌ Chyba: Cieľový adresár sa nenašiel: { $path }
bash-report-total-size =   💾 Celková veľkosť: { $size }

cli-help-usage = Použitie:
cli-help-options = Možnosti:
cli-help-print-help = Vypísať pomocník
cli-help-print-version = Vypísať verziu
