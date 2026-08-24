import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  applyStaticTranslations,
  availableLocales,
  currentLocale,
  formatBytes,
  formatCompact,
  formatCount,
  initI18n,
  localePreference,
  setLocale,
  t,
} from "./i18n";
import "./style.css";

interface ScanOptionsPayload {
  noGitignore: boolean;
  verbose: boolean;
  ignore: string[];
  maxOutputLines: number | null;
}

interface ScanOutcome {
  projectName: string;
  projectType: string;
  outputPaths: string[];
  processedFiles: number;
  skippedFiles: number;
  totalSizeHuman: string;
}

interface ExtensionStats {
  extension: string;
  language: string;
  files: number;
  lines: number;
  chars: number;
  estimatedTokens: number;
  size: number;
  sizeHuman: string;
}

interface AnalysisFileEntry {
  path: string;
  extension: string;
  lines: number;
  chars: number;
  estimatedTokens: number;
  size: number;
  sizeHuman: string;
}

interface SkippedReasonStats {
  reason: string;
  files: number;
}

interface TreeStats {
  directories: number;
  files: number;
  maxDepth: number;
}

interface AnalysisConfigState {
  configPresent: boolean;
  gitignoreEnabled: boolean;
  maxFileSize: number;
  maxFileSizeHuman: string;
}

interface ProjectAnalysis {
  projectName: string;
  projectType: string;
  rootPath: string;
  processedFiles: number;
  skippedFiles: number;
  totalLines: number;
  totalChars: number;
  estimatedTokens: number;
  totalSize: number;
  totalSizeHuman: string;
  extensionBreakdown: ExtensionStats[];
  largestByLines: AnalysisFileEntry[];
  largestByTokens: AnalysisFileEntry[];
  largestBySize: AnalysisFileEntry[];
  skippedReasons: SkippedReasonStats[];
  tree: TreeStats;
  config: AnalysisConfigState;
}

interface AnalysisSummary {
  processedFiles: number;
  skippedFiles: number;
  totalLines: number;
  estimatedTokens: number;
  totalSizeHuman: string;
}

interface HistoryRecord {
  path: string;
  projectName: string;
  projectType: string;
  analyzedAt: string;
  options: ScanOptionsPayload;
  summary: AnalysisSummary | null;
  outputPaths: string[];
}

type LargestMode = "tokens" | "lines" | "size";
type PresetId = "standard" | "verbose" | "split" | "heavy";

const THEME_KEY = "code-scanner-theme";
const HISTORY_KEY = "code-scanner-history-v2";
const MAX_HISTORY = 8;
const SPLIT_SAFE_LINES = 12000;
const CHART_COLORS = ["#32d4c1", "#ff7a59", "#f6c85f", "#7fd36b", "#63a6ff", "#c084fc", "#f472b6", "#94a3b8"];

const PRESETS: Record<PresetId, ScanOptionsPayload> = {
  standard: { noGitignore: false, verbose: false, ignore: [], maxOutputLines: null },
  verbose: { noGitignore: false, verbose: true, ignore: [], maxOutputLines: null },
  split: { noGitignore: false, verbose: false, ignore: [], maxOutputLines: SPLIT_SAFE_LINES },
  heavy: {
    noGitignore: false,
    verbose: false,
    ignore: ["png", "jpg", "jpeg", "gif", "svg", "zip", "tar", "gz", "pdf", "mp4", "mov", "ipynb"],
    maxOutputLines: null,
  },
};

const PRESET_I18N: Record<PresetId, string> = {
  standard: "ui-standard",
  verbose: "ui-verbose-review",
  split: "ui-split-safe",
  heavy: "ui-ignore-lock-heavy",
};

let currentAnalysis: ProjectAnalysis | null = null;
let largestMode: LargestMode = "tokens";
let historyRecords: HistoryRecord[] = [];

function byId<T extends Element>(id: string): T {
  const el = document.getElementById(id);
  if (!el) {
    throw new Error(`Missing element #${id}`);
  }
  return el as unknown as T;
}

function input(id: string): HTMLInputElement {
  return byId<HTMLInputElement>(id);
}

function parseIgnoreList(raw: string): string[] {
  return raw
    .split(/[\s,]+/)
    .map((part) => part.trim().replace(/^\./, ""))
    .filter(Boolean);
}

function suggestOutputDir(inputDir: string): string {
  const sep = inputDir.includes("\\") && !inputDir.includes("/") ? "\\" : "/";
  const trimmed = inputDir.replace(/[\\/]+$/, "");
  const parts = trimmed.split(sep);
  parts.pop();
  const parent = parts.join(sep) || sep;
  return `${parent}${sep}code-scanner-output`;
}

function getOptions(): ScanOptionsPayload {
  const maxRaw = input("max-lines-input").value;
  const maxOutputLines = maxRaw ? Number(maxRaw) : null;

  return {
    noGitignore: input("no-gitignore-option").checked,
    verbose: input("verbose-option").checked,
    ignore: parseIgnoreList(input("ignore-input").value),
    maxOutputLines: Number.isFinite(maxOutputLines) && maxOutputLines && maxOutputLines > 0 ? maxOutputLines : null,
  };
}

function applyOptions(options: ScanOptionsPayload): void {
  input("verbose-option").checked = options.verbose;
  input("no-gitignore-option").checked = options.noGitignore;
  input("ignore-input").value = options.ignore.join(" ");
  input("max-lines-input").value = options.maxOutputLines ? String(options.maxOutputLines) : "";
  syncPresetState(null);
}

function syncPresetState(active: PresetId | null): void {
  document.querySelectorAll<HTMLButtonElement>("[data-preset]").forEach((button) => {
    button.classList.toggle("is-active", button.dataset.preset === active);
  });
}

function setStatus(message: string): void {
  byId("status-pill").textContent = message;
}

function showError(message: string): void {
  const banner = byId<HTMLDivElement>("error-banner");
  banner.textContent = message;
  banner.hidden = false;
}

function hideError(): void {
  byId<HTMLDivElement>("error-banner").hidden = true;
}

function setLoading(kind: "analyze" | "scan", loading: boolean): void {
  const button = byId<HTMLButtonElement>(kind === "analyze" ? "analyze-project" : "run-scan");
  button.disabled = loading;
  button.classList.toggle("is-loading", loading);
}

function initTheme(): void {
  const saved = localStorage.getItem(THEME_KEY);
  if (saved === "light") {
    document.documentElement.dataset.theme = "light";
  }

  byId<HTMLButtonElement>("theme-toggle").addEventListener("click", () => {
    const isLight = document.documentElement.dataset.theme === "light";
    if (isLight) {
      delete document.documentElement.dataset.theme;
      localStorage.setItem(THEME_KEY, "dark");
    } else {
      document.documentElement.dataset.theme = "light";
      localStorage.setItem(THEME_KEY, "light");
    }
  });
}

function initTitlebarDrag(): void {
  const titlebar = document.querySelector<HTMLElement>(".titlebar");
  if (!titlebar) return;

  titlebar.addEventListener("mousedown", (event) => {
    if (event.button !== 0 || event.detail !== 1) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest("button, input, select, textarea, a, label, summary")) return;

    event.preventDefault();
    event.stopPropagation();
    getCurrentWindow().startDragging().catch(() => {});
  });
}

async function chooseFolder(): Promise<string | null> {
  const selection = await open({ directory: true, multiple: false });
  return typeof selection === "string" ? selection : null;
}

function setProjectPath(path: string): void {
  input("project-input").value = path;
  const output = input("output-input");
  if (!output.value.trim()) {
    output.value = suggestOutputDir(path);
  }
}

function renderAnalysis(analysis: ProjectAnalysis): void {
  byId("project-name").textContent = analysis.projectName || t("ui-unnamed-project");
  byId("project-type").textContent = analysis.projectType;
  byId("project-root").textContent = analysis.rootPath;
  byId("metric-files").textContent = formatCount(analysis.processedFiles);
  byId("metric-lines").textContent = formatCount(analysis.totalLines);
  byId("metric-tokens").textContent = formatCount(analysis.estimatedTokens);
  byId("metric-size").textContent = formatBytes(analysis.totalSize);
  byId("metric-skipped").textContent = formatCount(analysis.skippedFiles);
  byId("metric-depth").textContent = formatCount(analysis.tree.maxDepth);

  renderComposition(analysis);
  renderLargestFiles();
  renderReadiness(analysis, getOptions());
  renderDiagnostics(analysis);
}

function renderCachedRecord(record: HistoryRecord): void {
  byId("project-name").textContent = record.projectName || t("ui-recent-project");
  byId("project-type").textContent = record.projectType || t("ui-project-type-generic");
  byId("project-root").textContent = record.path;
  byId("metric-files").textContent = formatCount(record.summary?.processedFiles ?? 0);
  byId("metric-lines").textContent = formatCount(record.summary?.totalLines ?? 0);
  byId("metric-tokens").textContent = formatCount(record.summary?.estimatedTokens ?? 0);
  byId("metric-size").textContent = record.summary?.totalSizeHuman ?? t("ui-zero-bytes");
  byId("metric-skipped").textContent = formatCount(record.summary?.skippedFiles ?? 0);
  byId("metric-depth").textContent = formatCount(0);
  renderEmptyComposition(t("ui-analyze-to-refresh"));
  renderEmptyList("largest-list", t("ui-analyze-to-refresh-distribution"));
  renderEmptyList("readiness-list", t("ui-analyze-to-refresh-readiness"));
  renderEmptyList("skipped-list", t("ui-analyze-to-refresh-diagnostics"));
  renderReports(record.outputPaths);
}

function renderComposition(analysis: ProjectAnalysis): void {
  const entries = analysis.extensionBreakdown.slice(0, 8);
  const total = entries.reduce((sum, entry) => sum + entry.estimatedTokens, 0);
  byId("composition-kicker").textContent = t("ui-groups-count", {
    count: analysis.extensionBreakdown.length,
  });

  if (entries.length === 0 || total === 0) {
    renderEmptyComposition(t("ui-no-source-files"));
    return;
  }

  const svg = byId<SVGSVGElement>("extension-donut");
  svg.replaceChildren();

  const bg = document.createElementNS("http://www.w3.org/2000/svg", "circle");
  bg.setAttribute("cx", "50");
  bg.setAttribute("cy", "50");
  bg.setAttribute("r", "39");
  bg.setAttribute("class", "donut-bg");
  svg.appendChild(bg);

  const circumference = 2 * Math.PI * 39;
  let offset = 0;
  entries.forEach((entry, index) => {
    const segment = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    const length = (entry.estimatedTokens / total) * circumference;
    segment.setAttribute("cx", "50");
    segment.setAttribute("cy", "50");
    segment.setAttribute("r", "39");
    segment.setAttribute("class", "donut-segment");
    segment.setAttribute("stroke", CHART_COLORS[index % CHART_COLORS.length]);
    segment.setAttribute("stroke-dasharray", `${length} ${circumference - length}`);
    segment.setAttribute("stroke-dashoffset", String(-offset));
    segment.setAttribute("transform", "rotate(-90 50 50)");
    svg.appendChild(segment);
    offset += length;
  });

  const center = document.createElementNS("http://www.w3.org/2000/svg", "text");
  center.setAttribute("x", "50");
  center.setAttribute("y", "48");
  center.setAttribute("class", "donut-center");
  center.textContent = formatCompact(total);
  svg.appendChild(center);

  const caption = document.createElementNS("http://www.w3.org/2000/svg", "text");
  caption.setAttribute("x", "50");
  caption.setAttribute("y", "61");
  caption.setAttribute("class", "donut-caption");
  caption.textContent = t("ui-donut-tokens");
  svg.appendChild(caption);

  const legend = byId("extension-legend");
  legend.replaceChildren();
  entries.forEach((entry, index) => {
    const percent = Math.round((entry.estimatedTokens / total) * 100);
    const item = document.createElement("div");
    item.className = "legend-item";

    const swatch = document.createElement("span");
    swatch.className = "swatch";
    swatch.style.background = CHART_COLORS[index % CHART_COLORS.length];

    const label = document.createElement("span");
    label.className = "legend-name";
    label.textContent = `${entry.extension} - ${entry.language}`;

    const meta = document.createElement("span");
    meta.className = "legend-meta";
    meta.textContent = t("ui-legend-files", { percent, files: entry.files });

    item.append(swatch, label, meta);
    legend.appendChild(item);
  });
}

function renderEmptyComposition(message: string): void {
  const svg = byId<SVGSVGElement>("extension-donut");
  svg.replaceChildren();
  const bg = document.createElementNS("http://www.w3.org/2000/svg", "circle");
  bg.setAttribute("cx", "50");
  bg.setAttribute("cy", "50");
  bg.setAttribute("r", "39");
  bg.setAttribute("class", "donut-bg");
  svg.appendChild(bg);

  const center = document.createElementNS("http://www.w3.org/2000/svg", "text");
  center.setAttribute("x", "50");
  center.setAttribute("y", "54");
  center.setAttribute("class", "donut-center muted");
  center.textContent = "0";
  svg.appendChild(center);
  byId("extension-legend").replaceChildren(emptyMessage(message));
}

function renderLargestFiles(): void {
  if (!currentAnalysis) {
    renderEmptyList("largest-list", t("ui-analyze-to-populate"));
    return;
  }

  const entries =
    largestMode === "lines"
      ? currentAnalysis.largestByLines
      : largestMode === "size"
        ? currentAnalysis.largestBySize
        : currentAnalysis.largestByTokens;
  const max = Math.max(...entries.map((entry) => metricForEntry(entry, largestMode)), 1);
  const list = byId<HTMLOListElement>("largest-list");
  list.replaceChildren();

  if (entries.length === 0) {
    list.appendChild(emptyMessage(t("ui-no-files-matched")));
    return;
  }

  entries.forEach((entry, index) => {
    const value = metricForEntry(entry, largestMode);
    const li = document.createElement("li");
    const rank = document.createElement("span");
    rank.className = "rank";
    rank.textContent = String(index + 1);

    const body = document.createElement("div");
    body.className = "rank-body";
    const top = document.createElement("div");
    top.className = "rank-top";
    const path = document.createElement("span");
    path.className = "rank-path";
    path.textContent = entry.path;
    path.title = entry.path;
    const ext = document.createElement("span");
    ext.className = "mini-pill";
    ext.textContent = entry.extension;
    top.append(path, ext);

    const bar = document.createElement("div");
    bar.className = "bar-track";
    const fill = document.createElement("div");
    fill.className = "bar-fill";
    fill.style.width = `${Math.max(3, (value / max) * 100)}%`;
    bar.appendChild(fill);
    body.append(top, bar);

    const meta = document.createElement("span");
    meta.className = "rank-meta";
    meta.textContent = largestMode === "size" ? formatBytes(entry.size) : formatCount(value);

    li.append(rank, body, meta);
    list.appendChild(li);
  });
}

function metricForEntry(entry: AnalysisFileEntry, mode: LargestMode): number {
  if (mode === "lines") return entry.lines;
  if (mode === "size") return entry.size;
  return entry.estimatedTokens;
}

function renderReadiness(analysis: ProjectAnalysis, options: ScanOptionsPayload): void {
  const score = readinessScore(analysis, options);
  const title =
    analysis.estimatedTokens < 60000
      ? t("ui-readiness-compact")
      : analysis.estimatedTokens < 120000
        ? t("ui-readiness-review")
        : t("ui-readiness-large");

  byId("readiness-title").textContent = title;
  byId("readiness-score").textContent = String(score);
  const fill = byId<HTMLDivElement>("budget-fill");
  fill.style.width = `${score}%`;
  fill.dataset.score = score < 50 ? "low" : score < 75 ? "mid" : "high";

  const insights = readinessInsights(analysis, options);
  const list = byId("readiness-list");
  list.replaceChildren();
  insights.forEach((text) => {
    const li = document.createElement("li");
    li.textContent = text;
    list.appendChild(li);
  });
}

function readinessScore(analysis: ProjectAnalysis, options: ScanOptionsPayload): number {
  let score = 100;
  if (analysis.estimatedTokens > 60000) {
    score -= Math.min(45, Math.round((analysis.estimatedTokens - 60000) / 2500));
  }
  if (analysis.largestByTokens[0]?.estimatedTokens > 30000) {
    score -= 15;
  }
  if (analysis.skippedFiles > 0) {
    score -= Math.min(12, analysis.skippedFiles);
  }
  if (options.noGitignore) {
    score -= 12;
  }
  return Math.max(0, Math.min(100, score));
}

function readinessInsights(analysis: ProjectAnalysis, options: ScanOptionsPayload): string[] {
  const insights = [
    t("ui-insight-tokens-files", {
      tokens: formatCount(analysis.estimatedTokens),
      files: formatCount(analysis.processedFiles),
    }),
  ];

  if (options.maxOutputLines) {
    insights.push(
      t("ui-insight-split-active", { lines: formatCount(options.maxOutputLines) }),
    );
  } else if (analysis.totalLines > SPLIT_SAFE_LINES) {
    insights.push(t("ui-insight-split-suggest", { lines: formatCount(SPLIT_SAFE_LINES) }));
  } else {
    insights.push(t("ui-insight-split-optional"));
  }

  const topFile = analysis.largestByTokens[0];
  if (topFile) {
    insights.push(
      t("ui-insight-top-file", {
        path: topFile.path,
        tokens: formatCount(topFile.estimatedTokens),
      }),
    );
  }

  const topExtension = analysis.extensionBreakdown[0];
  if (topExtension && analysis.estimatedTokens > 0) {
    const share = Math.round((topExtension.estimatedTokens / analysis.estimatedTokens) * 100);
    insights.push(
      t("ui-insight-top-extension", { extension: topExtension.extension, percent: share }),
    );
  }

  if (options.noGitignore) {
    insights.push(t("ui-insight-gitignore-disabled"));
  }

  return insights;
}

function renderDiagnostics(analysis: ProjectAnalysis): void {
  byId("config-state").textContent = analysis.config.configPresent
    ? t("ui-project-config")
    : t("ui-defaults");
  byId("gitignore-state").textContent = analysis.config.gitignoreEnabled
    ? t("ui-enabled")
    : t("ui-disabled");
  byId("max-size-state").textContent = formatBytes(analysis.config.maxFileSize);

  const list = byId("skipped-list");
  list.replaceChildren();
  if (analysis.skippedReasons.length === 0) {
    list.appendChild(emptyMessage(t("ui-no-skipped-files")));
    return;
  }

  const max = Math.max(...analysis.skippedReasons.map((entry) => entry.files), 1);
  analysis.skippedReasons.forEach((entry) => {
    const li = document.createElement("li");
    const label = document.createElement("span");
    label.textContent = t(entry.reason);
    const count = document.createElement("strong");
    count.textContent = formatCount(entry.files);
    const bar = document.createElement("div");
    bar.className = "skip-bar";
    const fill = document.createElement("div");
    fill.style.width = `${Math.max(5, (entry.files / max) * 100)}%`;
    bar.appendChild(fill);
    li.append(label, count, bar);
    list.appendChild(li);
  });
}

function renderReports(paths: string[]): void {
  const list = byId("report-list");
  list.replaceChildren();

  if (paths.length === 0) {
    list.appendChild(emptyMessage(t("ui-no-saved-reports")));
    return;
  }

  paths.forEach((path) => {
    const li = document.createElement("li");
    const text = document.createElement("span");
    text.className = "report-path";
    text.textContent = path;
    text.title = path;

    const actions = document.createElement("div");
    actions.className = "report-actions";
    const reveal = document.createElement("button");
    reveal.type = "button";
    reveal.className = "icon-btn mini";
    reveal.title = t("ui-reveal-in-finder");
    reveal.innerHTML = folderIcon();
    reveal.addEventListener("click", () => {
      revealItemInDir(path).catch((err) => showError(String(err)));
    });

    const copy = document.createElement("button");
    copy.type = "button";
    copy.className = "icon-btn mini";
    copy.title = t("ui-copy-path");
    copy.innerHTML = copyIcon();
    copy.addEventListener("click", () => {
      navigator.clipboard
        .writeText(path)
        .then(() => setStatus(t("ui-status-report-copied")))
        .catch(() => showError(t("ui-error-copy-path")));
    });

    actions.append(reveal, copy);
    li.append(text, actions);
    list.appendChild(li);
  });
}

function renderEmptyList(id: string, message: string): void {
  byId(id).replaceChildren(emptyMessage(message));
}

function emptyMessage(message: string): HTMLElement {
  const el = document.createElement("div");
  el.className = "empty-message";
  el.textContent = message;
  return el;
}

function loadHistory(): HistoryRecord[] {
  try {
    const raw = localStorage.getItem(HISTORY_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as HistoryRecord[];
    return Array.isArray(parsed) ? parsed.slice(0, MAX_HISTORY) : [];
  } catch {
    return [];
  }
}

function saveHistory(): void {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(historyRecords.slice(0, MAX_HISTORY)));
}

function upsertHistory(record: HistoryRecord): void {
  historyRecords = [record, ...historyRecords.filter((item) => item.path !== record.path)].slice(0, MAX_HISTORY);
  saveHistory();
  renderHistory();
}

function summaryFromAnalysis(analysis: ProjectAnalysis): AnalysisSummary {
  return {
    processedFiles: analysis.processedFiles,
    skippedFiles: analysis.skippedFiles,
    totalLines: analysis.totalLines,
    estimatedTokens: analysis.estimatedTokens,
    totalSizeHuman: analysis.totalSizeHuman,
  };
}

function renderHistory(): void {
  const select = byId<HTMLSelectElement>("recent-projects");
  select.replaceChildren();
  if (historyRecords.length === 0) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = t("ui-no-recent-projects");
    select.appendChild(option);
  } else {
    const placeholder = document.createElement("option");
    placeholder.value = "";
    placeholder.textContent = t("ui-choose-recent-project");
    select.appendChild(placeholder);
    historyRecords.forEach((record) => {
      const option = document.createElement("option");
      option.value = record.path;
      option.textContent = record.projectName || record.path;
      select.appendChild(option);
    });
  }

  const list = byId("history-list");
  list.replaceChildren();
  if (historyRecords.length === 0) {
    list.appendChild(emptyMessage(t("ui-no-local-history")));
    return;
  }

  historyRecords.forEach((record) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "history-item";
    button.addEventListener("click", () => loadHistoryRecord(record));

    const name = document.createElement("strong");
    name.textContent = record.projectName || record.path;
    const meta = document.createElement("span");
    const tokens = record.summary
      ? t("ui-history-tokens", { tokens: formatCompact(record.summary.estimatedTokens) })
      : t("ui-not-analyzed");
    meta.textContent = `${record.projectType || t("ui-project-type-generic")} - ${tokens}`;
    const path = document.createElement("small");
    path.textContent = record.path;
    button.append(name, meta, path);
    list.appendChild(button);
  });
}

function loadHistoryRecord(record: HistoryRecord): void {
  currentAnalysis = null;
  setProjectPath(record.path);
  applyOptions(record.options);
  renderCachedRecord(record);
  setStatus(t("ui-status-recent-loaded"));
}

async function handleAnalyze(): Promise<void> {
  const path = input("project-input").value.trim();
  if (!path) {
    showError(t("ui-error-choose-folder-analyze"));
    return;
  }

  hideError();
  setLoading("analyze", true);
  setStatus(t("ui-status-analyzing"));

  try {
    const options = getOptions();
    const analysis = await invoke<ProjectAnalysis>("analyze_project", { path, options });
    currentAnalysis = analysis;
    renderAnalysis(analysis);
    const previousOutputs = historyRecords.find((record) => record.path === analysis.rootPath)?.outputPaths ?? [];
    renderReports(previousOutputs);
    upsertHistory({
      path: analysis.rootPath,
      projectName: analysis.projectName,
      projectType: analysis.projectType,
      analyzedAt: new Date().toISOString(),
      options,
      summary: summaryFromAnalysis(analysis),
      outputPaths: previousOutputs,
    });
    setStatus(t("ui-status-analyzed-files", { count: formatCount(analysis.processedFiles) }));
  } catch (err) {
    showError(typeof err === "string" ? err : t("ui-error-analyze"));
    setStatus(t("ui-status-analysis-failed"));
  } finally {
    setLoading("analyze", false);
  }
}

async function handleRunScan(): Promise<void> {
  const inputDir = input("project-input").value.trim();
  if (!inputDir) {
    showError(t("ui-error-choose-folder-scan"));
    return;
  }

  const outputInput = input("output-input");
  if (!outputInput.value.trim()) {
    outputInput.value = suggestOutputDir(inputDir);
  }

  hideError();
  setLoading("scan", true);
  setStatus(t("ui-status-scanning"));

  try {
    const outputDir = outputInput.value.trim();
    const options = getOptions();
    const outcome = await invoke<ScanOutcome>("run_scan", { inputDir, outputDir, options });
    renderReports(outcome.outputPaths);
    const existing = historyRecords.find((record) => record.path === inputDir);
    upsertHistory({
      path: inputDir,
      projectName: outcome.projectName,
      projectType: outcome.projectType,
      analyzedAt: new Date().toISOString(),
      options,
      summary: currentAnalysis ? summaryFromAnalysis(currentAnalysis) : existing?.summary ?? null,
      outputPaths: outcome.outputPaths,
    });
    setStatus(t("ui-status-saved-reports", { count: outcome.outputPaths.length }));
  } catch (err) {
    showError(typeof err === "string" ? err : t("ui-error-scan"));
    setStatus(t("ui-status-scan-failed"));
  } finally {
    setLoading("scan", false);
  }
}

function bindEvents(): void {
  byId<HTMLButtonElement>("browse-project").addEventListener("click", async () => {
    const folder = await chooseFolder();
    if (!folder) return;
    setProjectPath(folder);
    setStatus(t("ui-status-project-selected"));
  });

  byId<HTMLButtonElement>("browse-output").addEventListener("click", async () => {
    const folder = await chooseFolder();
    if (!folder) return;
    input("output-input").value = folder;
  });

  byId<HTMLButtonElement>("analyze-project").addEventListener("click", () => {
    void handleAnalyze();
  });
  byId<HTMLButtonElement>("run-scan").addEventListener("click", () => {
    void handleRunScan();
  });

  byId<HTMLSelectElement>("recent-projects").addEventListener("change", (event) => {
    const value = (event.currentTarget as HTMLSelectElement).value;
    const record = historyRecords.find((item) => item.path === value);
    if (record) loadHistoryRecord(record);
  });

  document.querySelectorAll<HTMLButtonElement>("[data-preset]").forEach((button) => {
    button.addEventListener("click", () => {
      const preset = button.dataset.preset as PresetId;
      applyOptions(PRESETS[preset]);
      syncPresetState(preset);
      setStatus(t("ui-status-preset-applied", { preset: t(PRESET_I18N[preset]) }));
      if (currentAnalysis) {
        renderReadiness(currentAnalysis, getOptions());
      }
    });
  });

  document.querySelectorAll<HTMLButtonElement>("[data-largest-mode]").forEach((button) => {
    button.addEventListener("click", () => {
      largestMode = button.dataset.largestMode as LargestMode;
      document.querySelectorAll<HTMLButtonElement>("[data-largest-mode]").forEach((item) => {
        item.classList.toggle("is-active", item === button);
      });
      renderLargestFiles();
    });
  });

  ["verbose-option", "no-gitignore-option", "ignore-input", "max-lines-input"].forEach((id) => {
    byId<HTMLInputElement>(id).addEventListener("input", () => {
      syncPresetState(null);
      if (currentAnalysis) {
        renderReadiness(currentAnalysis, getOptions());
      }
    });
  });
}

function folderIcon(): string {
  return '<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 5h6l2 2h8v12H4V5Z" stroke-linejoin="round"/></svg>';
}

function copyIcon(): string {
  return '<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="8" y="8" width="10" height="12" rx="2"/><path d="M6 16H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v1" stroke-linejoin="round"/></svg>';
}

function renderInitialState(): void {
  applyOptions(PRESETS.standard);
  syncPresetState("standard");
  byId("project-name").textContent = t("ui-no-project-selected");
  byId("project-root").textContent = t("ui-choose-folder");
  byId("project-type").textContent = t("ui-project-type-generic");
  byId("config-state").textContent = t("ui-defaults");
  byId("gitignore-state").textContent = t("ui-enabled");
  setStatus(t("ui-idle"));
  renderEmptyComposition(t("ui-no-analysis-yet"));
  renderEmptyList("largest-list", t("ui-analyze-to-populate"));
  renderEmptyList("readiness-list", t("ui-analyze-to-calculate-readiness"));
  renderEmptyList("skipped-list", t("ui-analyze-to-inspect-skipped"));
  byId("readiness-title").textContent = t("ui-awaiting-analysis");
  renderReports([]);
  historyRecords = loadHistory();
  renderHistory();
}

function localeLabel(tag: string): string {
  try {
    return new Intl.DisplayNames([currentLocale(), tag], { type: "language" }).of(tag) ?? tag;
  } catch {
    return tag;
  }
}

async function syncRustLocale(preference: string | null): Promise<void> {
  try {
    await invoke("set_ui_locale", {
      locale: !preference || preference === "system" ? null : preference,
    });
  } catch {
    // Browser preview without Tauri.
  }
}

function populateLanguageSelect(): void {
  const select = byId<HTMLSelectElement>("language-select");
  select.replaceChildren();
  const system = document.createElement("option");
  system.value = "system";
  system.textContent = t("ui-language-system");
  select.appendChild(system);
  availableLocales().forEach((locale) => {
    const option = document.createElement("option");
    option.value = locale;
    option.textContent = localeLabel(locale);
    select.appendChild(option);
  });
  const stored = localePreference();
  select.value = stored && stored !== "system" ? stored : "system";
}

function rerenderForLocale(): void {
  applyStaticTranslations();
  populateLanguageSelect();
  if (currentAnalysis) {
    renderAnalysis(currentAnalysis);
    const outputs =
      historyRecords.find((record) => record.path === currentAnalysis?.rootPath)?.outputPaths ?? [];
    renderReports(outputs);
  } else {
    const path = input("project-input").value.trim();
    const record = historyRecords.find((item) => item.path === path);
    if (record) {
      renderCachedRecord(record);
    } else {
      byId("project-name").textContent = t("ui-no-project-selected");
      byId("project-root").textContent = t("ui-choose-folder");
      byId("project-type").textContent = t("ui-project-type-generic");
      byId("config-state").textContent = t("ui-defaults");
      byId("gitignore-state").textContent = t("ui-enabled");
      renderEmptyComposition(t("ui-no-analysis-yet"));
      renderEmptyList("largest-list", t("ui-analyze-to-populate"));
      renderEmptyList("readiness-list", t("ui-analyze-to-calculate-readiness"));
      renderEmptyList("skipped-list", t("ui-analyze-to-inspect-skipped"));
      byId("readiness-title").textContent = t("ui-awaiting-analysis");
    }
  }
  renderHistory();
  const status = byId("status-pill").textContent;
  if (!status || status === "Idle" || status === t("ui-idle")) {
    setStatus(t("ui-idle"));
  }
}

function initLanguage(): void {
  initI18n();
  populateLanguageSelect();
  void syncRustLocale(localePreference());
  byId<HTMLSelectElement>("language-select").addEventListener("change", (event) => {
    const value = (event.currentTarget as HTMLSelectElement).value;
    setLocale(value);
    void syncRustLocale(value);
    rerenderForLocale();
  });
}

function main(): void {
  initLanguage();
  initTheme();
  initTitlebarDrag();
  renderInitialState();
  bindEvents();
}

main();
