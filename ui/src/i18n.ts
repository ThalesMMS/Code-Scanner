import { FluentBundle, FluentResource } from "@fluent/bundle";

const LANG_KEY = "code-scanner-lang";
const FALLBACK = "en-US";

const catalogModules = import.meta.glob("../../i18n/*.ftl", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

export type FluentVars = Record<string, string | number>;

const catalogs = new Map<string, string>();
for (const [path, source] of Object.entries(catalogModules)) {
  const file = path.split("/").pop() ?? "";
  if (!file.endsWith(".ftl")) continue;
  catalogs.set(file.slice(0, -4), source);
}

let bundle = new FluentBundle(FALLBACK, { useIsolating: false });
let activeLocale = FALLBACK;
let preference: string | null = null;

function osLocale(): string {
  return navigator.language || FALLBACK;
}

export function availableLocales(): string[] {
  const locales = [...catalogs.keys()].sort();
  const fallbackIndex = locales.indexOf(FALLBACK);
  if (fallbackIndex > 0) {
    locales.splice(fallbackIndex, 1);
    locales.unshift(FALLBACK);
  }
  return locales;
}

const RTL_LANGUAGES = new Set(["ar", "he", "fa", "ur"]);

export function isRtlLocale(tag: string): boolean {
  const lang = tag.split(/[-_]/)[0]?.toLowerCase() ?? tag.toLowerCase();
  return RTL_LANGUAGES.has(lang);
}

export function matchLocale(requested: string): string {
  const normalized = requested.replaceAll("_", "-");
  const locales = availableLocales();
  if (locales.includes(normalized)) return normalized;
  const lang = normalized.split("-")[0]?.toLowerCase();
  const close = locales.find((item) => item.split("-")[0]?.toLowerCase() === lang);
  return close ?? FALLBACK;
}

export function resolveLocale(stored: string | null): string {
  if (stored && stored !== "system") {
    return matchLocale(stored);
  }
  return matchLocale(osLocale());
}

function buildBundle(locale: string): FluentBundle {
  const next = new FluentBundle(locale, { useIsolating: false });
  const localeSrc = catalogs.get(locale);
  const fallbackSrc = catalogs.get(FALLBACK) ?? "";
  if (localeSrc && locale !== FALLBACK) {
    next.addResource(new FluentResource(localeSrc));
  }
  next.addResource(new FluentResource(fallbackSrc));
  return next;
}

export function currentLocale(): string {
  return activeLocale;
}

export function localePreference(): string | null {
  return preference;
}

export function t(id: string, vars?: FluentVars): string {
  const message = bundle.getMessage(id);
  if (!message?.value) return id;
  const args = vars ? { ...vars } : undefined;
  return bundle.formatPattern(message.value, args);
}

export function applyStaticTranslations(root: ParentNode = document): void {
  root.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    const id = el.dataset.i18n;
    if (!id) return;
    const text = t(id);
    const target = el.querySelector("[data-i18n-text]") ?? el;
    target.textContent = text;
  });
  root.querySelectorAll<HTMLElement>("[data-i18n-placeholder]").forEach((el) => {
    const id = el.dataset.i18nPlaceholder;
    if (id && "placeholder" in el) {
      (el as HTMLInputElement).placeholder = t(id);
    }
  });
  root.querySelectorAll<HTMLElement>("[data-i18n-title]").forEach((el) => {
    const id = el.dataset.i18nTitle;
    if (id) el.title = t(id);
  });
  root.querySelectorAll<HTMLElement>("[data-i18n-aria-label]").forEach((el) => {
    const id = el.dataset.i18nAriaLabel;
    if (id) el.setAttribute("aria-label", t(id));
  });
}

export function formatCount(value: number): string {
  return new Intl.NumberFormat(activeLocale).format(value);
}

export function formatCompact(value: number): string {
  return new Intl.NumberFormat(activeLocale, {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(value);
}

export function formatBytes(bytes: number): string {
  const units = ["byte", "kilobyte", "megabyte", "gigabyte", "terabyte"];
  let unitIndex = 0;
  let amount = bytes;
  while (amount >= 1000 && unitIndex < units.length - 1) {
    amount /= 1000;
    unitIndex += 1;
  }
  return new Intl.NumberFormat(activeLocale, {
    style: "unit",
    unit: units[unitIndex],
    unitDisplay: "short",
    maximumFractionDigits: unitIndex === 0 ? 0 : 1,
  }).format(amount);
}

export function formatDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return new Intl.DateTimeFormat(activeLocale, { dateStyle: "medium", timeStyle: "short" }).format(date);
}

export function setLocale(nextPreference: string | null, persist = true): string {
  preference = nextPreference;
  if (persist) {
    if (!nextPreference || nextPreference === "system") {
      localStorage.setItem(LANG_KEY, "system");
    } else {
      localStorage.setItem(LANG_KEY, nextPreference);
    }
  }
  activeLocale = resolveLocale(nextPreference);
  bundle = buildBundle(activeLocale);
  document.documentElement.lang = activeLocale;
  document.documentElement.dir = isRtlLocale(activeLocale) ? "rtl" : "ltr";
  applyStaticTranslations();
  return activeLocale;
}

export function initI18n(): string {
  const saved = localStorage.getItem(LANG_KEY);
  return setLocale(saved === null ? "system" : saved, false);
}

export { LANG_KEY };
