//
// i18n.rs
// Code-Scanner-rs
//
// Embeds Fluent catalogs from i18n/*.ftl and resolves CLI/desktop locale.
//
// Thales Matheus Mendonça Santos - August 2026
//

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use rust_embed::RustEmbed;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};
use unic_langid::LanguageIdentifier;

const FALLBACK_LOCALE: &str = "en-US";
const ENV_LANG: &str = "CODE_SCANNER_LANG";
const RTL_LANGUAGES: &[&str] = &["ar", "he", "fa", "ur"];

#[derive(RustEmbed)]
#[folder = "i18n/"]
#[include = "*.ftl"]
struct Catalogs;

static LOCALE: OnceLock<Mutex<String>> = OnceLock::new();

thread_local! {
    static BUNDLE: RefCell<Option<(String, I18n)>> = RefCell::new(None);
}

pub struct I18n {
    pub locale: LanguageIdentifier,
    bundle: FluentBundle<FluentResource>,
}

pub fn init(requested: Option<&str>) {
    let locale = resolve_locale(requested);
    *locale_lock() = locale.to_string();
    BUNDLE.with(|slot| {
        *slot.borrow_mut() = Some((locale.to_string(), I18n::load(&locale)));
    });
}

/// `--lang` → `CODE_SCANNER_LANG` → OS locale → `en-US`.
pub fn init_from_cli() {
    init(peek_argv_lang().as_deref());
}

pub fn peek_argv_lang() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--lang" {
            return args.next().filter(|s| !s.is_empty());
        }
        if let Some(value) = arg.strip_prefix("--lang=") {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

pub fn current_locale() -> String {
    locale_lock().clone()
}

pub fn is_rtl() -> bool {
    is_rtl_locale(&current_locale())
}

pub fn is_rtl_locale(tag: &str) -> bool {
    let lang = tag
        .split(['-', '_'])
        .next()
        .unwrap_or(tag)
        .to_ascii_lowercase();
    RTL_LANGUAGES.contains(&lang.as_str())
}

pub fn format_number(value: i64) -> String {
    localize_int(value)
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{} {}", format_number(bytes as i64), UNITS[0]);
    }

    let mut amount = bytes as f64;
    let mut unit = 0usize;
    while amount >= 1000.0 && unit + 1 < UNITS.len() {
        amount /= 1000.0;
        unit += 1;
    }
    format!("{} {}", localize_float(amount), UNITS[unit])
}

fn locale_decimal_separator() -> char {
    let lang = LanguageIdentifier::from_str(&current_locale())
        .map(|id| id.language.to_string())
        .unwrap_or_else(|_| "en".into());
    match lang.as_str() {
        "en" | "ja" | "ko" | "zh" | "th" | "he" | "id" | "ms" | "fil" | "hi" | "ta" | "te"
        | "mr" | "kn" | "gu" | "pa" | "ml" | "bn" | "sw" | "am" | "my" | "lo" | "km" => '.',
        _ => ',',
    }
}

fn locale_thousands_separator() -> char {
    if locale_decimal_separator() == ',' {
        '.'
    } else {
        ','
    }
}

fn localize_int(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let thousands = locale_thousands_separator();
    let mut grouped = String::new();
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(thousands);
        }
        grouped.push(ch);
    }
    let mut out: String = grouped.chars().rev().collect();
    if negative {
        out.insert(0, '-');
    }
    out
}

fn localize_float(amount: f64) -> String {
    let mut text = format!("{amount:.2}");
    if text.contains('.') {
        text = text
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
    }
    let decimal = locale_decimal_separator();
    if decimal != '.' {
        text = text.replace('.', &decimal.to_string());
    }
    text
}

pub fn format_timestamp(now: chrono::DateTime<chrono::Local>) -> String {
    let tag = current_locale().replace('-', "_");
    if let Ok(locale) = chrono::Locale::try_from(tag.as_str()) {
        return now.format_localized("%x %X", locale).to_string();
    }
    if let Some(lang) = tag.split('_').next() {
        if let Ok(locale) = chrono::Locale::try_from(lang) {
            return now.format_localized("%x %X", locale).to_string();
        }
    }
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn available_locales() -> Vec<String> {
    let mut locales: Vec<String> = catalog_map().into_keys().collect();
    locales.sort();
    if let Some(index) = locales.iter().position(|l| l == FALLBACK_LOCALE) {
        locales.remove(index);
        locales.insert(0, FALLBACK_LOCALE.to_string());
    }
    locales
}

pub fn catalog_source(locale: &str) -> Option<String> {
    catalog_map().get(locale).cloned()
}

pub fn all_catalogs() -> BTreeMap<String, String> {
    catalog_map()
}

pub fn translate(id: &str, args: Option<&FluentArgs>) -> String {
    let locale = current_locale();
    BUNDLE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let needs_load = match slot.as_ref() {
            Some((active, _)) => active != &locale,
            None => true,
        };
        if needs_load {
            let ident = LanguageIdentifier::from_str(&locale).unwrap_or_else(|_| lang_id(FALLBACK_LOCALE));
            *slot = Some((locale.clone(), I18n::load(&ident)));
        }
        slot.as_ref()
            .expect("i18n bundle")
            .1
            .format(id, args)
    })
}

pub enum ArgVal {
    Str(String),
    Int(i64),
    Float(f64),
}

impl From<String> for ArgVal {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<&str> for ArgVal {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

impl From<i64> for ArgVal {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<u64> for ArgVal {
    fn from(value: u64) -> Self {
        Self::Int(value as i64)
    }
}

impl From<usize> for ArgVal {
    fn from(value: usize) -> Self {
        Self::Int(value as i64)
    }
}

impl From<f64> for ArgVal {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

pub fn translate_owned(id: &str, pairs: Vec<(&'static str, ArgVal)>) -> String {
    let mut args = FluentArgs::new();
    for (key, value) in &pairs {
        match value {
            ArgVal::Str(text) => args.set(*key, text.as_str()),
            ArgVal::Int(number) => args.set(*key, *number),
            ArgVal::Float(number) => args.set(*key, *number),
        }
    }
    translate(id, Some(&args))
}

pub fn message_ids(src: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for line in src.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('.') {
            continue;
        }
        let Some((raw_id, _)) = trimmed.split_once('=') else {
            continue;
        };
        let id = raw_id.trim();
        if id.starts_with('-') {
            continue;
        }
        if !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            ids.insert(id.to_string());
        }
    }
    ids
}

impl I18n {
    fn load(locale: &LanguageIdentifier) -> Self {
        let catalogs = catalog_map();
        let fallback_src = catalogs
            .get(FALLBACK_LOCALE)
            .cloned()
            .unwrap_or_default();
        let locale_tag = locale.to_string();
        let locale_src = catalogs.get(&locale_tag).cloned();

        let mut bundle = FluentBundle::new(vec![locale.clone()]);
        bundle.set_use_isolating(false);
        if let Some(src) = locale_src {
            if locale_tag != FALLBACK_LOCALE {
                add_resource(&mut bundle, &src);
            }
        }
        add_resource(&mut bundle, &fallback_src);

        Self {
            locale: locale.clone(),
            bundle,
        }
    }

    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        let Some(message) = self.bundle.get_message(id) else {
            return id.to_string();
        };
        let Some(pattern) = message.value() else {
            return id.to_string();
        };
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, args, &mut errors)
            .to_string()
    }
}

fn locale_lock() -> std::sync::MutexGuard<'static, String> {
    LOCALE
        .get_or_init(|| Mutex::new(resolve_locale(None).to_string()))
        .lock()
        .expect("i18n locale lock")
}

fn add_resource(bundle: &mut FluentBundle<FluentResource>, src: &str) {
    if src.trim().is_empty() {
        return;
    }
    let resource = match FluentResource::try_new(src.to_string()) {
        Ok(resource) => resource,
        Err((resource, _)) => resource,
    };
    let _ = bundle.add_resource(resource);
}

fn catalog_map() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for file in Catalogs::iter() {
        let name = file.as_ref();
        if !name.ends_with(".ftl") {
            continue;
        }
        let locale = name.trim_end_matches(".ftl").replace('_', "-");
        let Some(bytes) = Catalogs::get(name) else {
            continue;
        };
        let Ok(text) = std::str::from_utf8(bytes.data.as_ref()) else {
            continue;
        };
        map.insert(locale, text.to_string());
    }
    map
}

pub fn resolve_locale(requested: Option<&str>) -> LanguageIdentifier {
    let available = available_locale_ids();
    let fallback = lang_id(FALLBACK_LOCALE);

    if let Some(tag) = requested.and_then(clean_tag) {
        if let Some(matched) = match_available(&tag, &available) {
            return matched;
        }
    }

    if let Ok(tag) = std::env::var(ENV_LANG) {
        if let Some(tag) = clean_tag(&tag) {
            if let Some(matched) = match_available(&tag, &available) {
                return matched;
            }
        }
    }

    if let Some(os) = sys_locale::get_locale().and_then(|s| clean_tag(&s)) {
        if let Some(matched) = match_available(&os, &available) {
            return matched;
        }
    }

    fallback
}

fn available_locale_ids() -> Vec<LanguageIdentifier> {
    catalog_map()
        .into_keys()
        .filter_map(|tag| LanguageIdentifier::from_str(&tag).ok())
        .collect()
}

fn match_available(
    requested: &str,
    available: &[LanguageIdentifier],
) -> Option<LanguageIdentifier> {
    let requested = LanguageIdentifier::from_str(requested).ok()?;
    if let Some(exact) = available.iter().find(|id| *id == &requested) {
        return Some(exact.clone());
    }
    available
        .iter()
        .find(|id| id.language == requested.language)
        .cloned()
}

fn lang_id(tag: &str) -> LanguageIdentifier {
    LanguageIdentifier::from_str(tag).unwrap_or_else(|_| "en-US".parse().expect("en-US"))
}

fn clean_tag(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.replace('_', "-"))
}

#[macro_export]
macro_rules! t {
    ($id:expr) => {
        $crate::i18n::translate($id, None)
    };
    ($id:expr, $($key:expr => $value:expr),+ $(,)?) => {
        $crate::i18n::translate_owned(
            $id,
            vec![$(($key, $crate::i18n::ArgVal::from($value))),+],
        )
    };
}
