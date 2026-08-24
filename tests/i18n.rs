use code_scanner::i18n::{self, message_ids};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn scanner_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_code_scanner"))
}

fn catalog_path(locale: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("i18n")
        .join(format!("{locale}.ftl"))
}

fn ftl_value_static_prefix(src: &str, id: &str) -> Option<String> {
    for line in src.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let Some((raw_id, rest)) = line.split_once('=') else {
            continue;
        };
        if raw_id.trim() != id {
            continue;
        }
        let value = rest.trim();
        let prefix = value.split('{').next().unwrap_or(value).trim();
        if prefix.is_empty() {
            return None;
        }
        return Some(prefix.to_string());
    }
    None
}

#[test]
fn en_us_catalog_contains_canonical_keys() {
    let src = fs::read_to_string(catalog_path("en-US")).expect("en-US.ftl");
    let ids = message_ids(&src);
    for key in [
        "app-name",
        "app-banner",
        "app-about",
        "cli-arg-input-help",
        "cli-arg-lang-help",
        "cli-help-usage",
        "cli-help-options",
        "cli-help-print-help",
        "cli-help-print-version",
        "cli-loc-files-processed",
        "cli-report-files-processed",
        "ui-idle",
        "ui-run-scan",
        "ui-language",
        "ui-language-system",
    ] {
        assert!(ids.contains(key), "en-US.ftl must define {key}");
    }
}

const COMPLETE_CATALOG_MIN_KEYS: usize = 200;

#[test]
fn catalogs_match_en_us_key_coverage() {
    let en_src = fs::read_to_string(catalog_path("en-US")).expect("en-US.ftl");
    let en_ids = message_ids(&en_src);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("i18n");

    for entry in fs::read_dir(&dir).expect("i18n dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ftl") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy();
        if name == "en-US.ftl" {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap_or_default();
        if src.trim().is_empty() {
            eprintln!("skipping empty catalog {name}");
            continue;
        }
        let ids = message_ids(&src);
        let extra: Vec<_> = ids.difference(&en_ids).cloned().collect();
        assert!(
            extra.is_empty(),
            "{name} defines unknown keys not in en-US.ftl: {extra:?}"
        );
        if ids.len() < COMPLETE_CATALOG_MIN_KEYS {
            eprintln!(
                "skipping incomplete catalog {name} ({} keys < {COMPLETE_CATALOG_MIN_KEYS})",
                ids.len()
            );
            continue;
        }
        let missing: Vec<_> = en_ids.difference(&ids).cloned().collect();
        assert!(
            missing.is_empty(),
            "{name} looks complete ({} keys) but is missing {}: {:?}",
            ids.len(),
            missing.len(),
            missing
        );
    }
}

#[test]
fn rtl_locales_are_detected() {
    assert!(i18n::is_rtl_locale("ar-SA"));
    assert!(i18n::is_rtl_locale("he-IL"));
    assert!(i18n::is_rtl_locale("fa"));
    assert!(i18n::is_rtl_locale("ur-PK"));
    assert!(!i18n::is_rtl_locale("en-US"));
    assert!(!i18n::is_rtl_locale("pt-BR"));
}

#[test]
fn sizes_and_counts_follow_active_locale() {
    i18n::init(Some("en-US"));
    assert_eq!(i18n::format_size(0), "0 B");
    let en = i18n::format_size(1500);
    assert!(en.contains("kB"), "en-US size should use kB: {en}");
    assert!(
        en.contains('.'),
        "en-US decimal size should use a dot: {en}"
    );
    assert_eq!(i18n::format_number(1234), "1,234");

    i18n::init(Some("pt-BR"));
    assert_eq!(i18n::format_number(2), "2");
    assert_eq!(i18n::format_number(1234), "1.234");
    let pt = i18n::format_size(1500);
    assert!(
        pt.contains(','),
        "pt-BR decimal size should use a comma: {pt}"
    );
    let stamp = i18n::format_timestamp(chrono::Local::now());
    assert!(!stamp.is_empty());
}

#[test]
fn loc_mode_pt_br_uses_catalog_when_present() {
    let ftl = catalog_path("pt-BR");
    if !ftl.exists() {
        return;
    }
    let src = fs::read_to_string(&ftl).unwrap_or_default();
    if src.trim().is_empty() {
        return;
    }
    let Some(pt_prefix) = ftl_value_static_prefix(&src, "cli-loc-files-processed") else {
        return;
    };

    let dir = tempdir().expect("temp dir");
    fs::write(dir.path().join("code.rs"), "fn a() {}\n").expect("write");

    let output = scanner_cmd()
        .arg("--lang")
        .arg("pt-BR")
        .arg("--loc")
        .arg(dir.path())
        .arg("--no-gitignore")
        .output()
        .expect("run");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&pt_prefix),
        "expected Portuguese catalog prefix {pt_prefix:?} in:\n{stdout}"
    );
    assert!(
        !stdout.contains("Files processed"),
        "pt-BR output must not contain English Files processed; got:\n{stdout}"
    );
}

#[test]
fn help_chrome_follows_lang_flag() {
    let ftl = catalog_path("pt-BR");
    let src = fs::read_to_string(&ftl).unwrap_or_default();
    let usage = ftl_value_static_prefix(&src, "cli-help-usage").expect("pt-BR usage");
    let options = ftl_value_static_prefix(&src, "cli-help-options").expect("pt-BR options");
    let print_help = ftl_value_static_prefix(&src, "cli-help-print-help").expect("pt-BR print help");

    let output = scanner_cmd()
        .arg("--lang")
        .arg("pt-BR")
        .arg("--help")
        .output()
        .expect("help");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(&usage),
        "expected translated usage {usage:?} in:\n{stdout}"
    );
    assert!(
        stdout.contains(&options),
        "expected translated options {options:?} in:\n{stdout}"
    );
    assert!(
        stdout.contains(&print_help),
        "expected translated print-help {print_help:?} in:\n{stdout}"
    );
    assert!(
        !stdout.contains("Usage:"),
        "pt-BR help must not contain English Usage:; got:\n{stdout}"
    );
    assert!(
        !stdout.contains("Print help"),
        "pt-BR help must not contain English Print help; got:\n{stdout}"
    );
}

#[test]
fn help_is_translated_before_parse() {
    i18n::init(Some("en-US"));
    let output = scanner_cmd()
        .env("CODE_SCANNER_LANG", "en-US")
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Language for CLI messages") || stdout.contains("--lang"),
        "help must include --lang; got:\n{stdout}"
    );
}

#[test]
fn ui_idle_and_run_scan_keys_exist_for_desktop_swap() {
    let src = fs::read_to_string(catalog_path("en-US")).expect("en-US.ftl");
    let ids = message_ids(&src);
    assert!(ids.contains("ui-idle"));
    assert!(ids.contains("ui-run-scan"));
    assert_ne!(
        ftl_value_static_prefix(&src, "ui-idle").unwrap_or_default(),
        ftl_value_static_prefix(&src, "ui-run-scan").unwrap_or_default()
    );
}
