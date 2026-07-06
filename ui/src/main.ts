import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
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

function formatCount(value: number): string {
  return value.toLocaleString();
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
  byId("project-name").textContent = analysis.projectName || "Unnamed project";
  byId("project-type").textContent = analysis.projectType;
  byId("project-root").textContent = analysis.rootPath;
  byId("metric-files").textContent = formatCount(analysis.processedFiles);
  byId("metric-lines").textContent = formatCount(analysis.totalLines);
  byId("metric-tokens").textContent = formatCount(analysis.estimatedTokens);
  byId("metric-size").textContent = analysis.totalSizeHuman;
  byId("metric-skipped").textContent = formatCount(analysis.skippedFiles);
  byId("metric-depth").textContent = formatCount(analysis.tree.maxDepth);

  renderComposition(analysis);
  renderLargestFiles();
  renderReadiness(analysis, getOptions());
  renderDiagnostics(analysis);
}

function renderCachedRecord(record: HistoryRecord): void {
  byId("project-name").textContent = record.projectName || "Recent project";
  byId("project-type").textContent = record.projectType || "Generic";
  byId("project-root").textContent = record.path;
  byId("metric-files").textContent = formatCount(record.summary?.processedFiles ?? 0);
  byId("metric-lines").textContent = formatCount(record.summary?.totalLines ?? 0);
  byId("metric-tokens").textContent = formatCount(record.summary?.estimatedTokens ?? 0);
  byId("metric-size").textContent = record.summary?.totalSizeHuman ?? "0 B";
  byId("metric-skipped").textContent = formatCount(record.summary?.skippedFiles ?? 0);
  byId("metric-depth").textContent = "0";
  renderEmptyComposition("Analyze to refresh");
  renderEmptyList("largest-list", "Analyze to refresh file distribution.");
  renderEmptyList("readiness-list", "Analyze to refresh readiness.");
  renderEmptyList("skipped-list", "Analyze to refresh diagnostics.");
  renderReports(record.outputPaths);
}

function renderComposition(analysis: ProjectAnalysis): void {
  const entries = analysis.extensionBreakdown.slice(0, 8);
  const total = entries.reduce((sum, entry) => sum + entry.estimatedTokens, 0);
  byId("composition-kicker").textContent = `${analysis.extensionBreakdown.length.toLocaleString()} groups`;

  if (entries.length === 0 || total === 0) {
    renderEmptyComposition("No source files");
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
  caption.textContent = "tokens";
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
    meta.textContent = `${percent}% - ${entry.files.toLocaleString()} files`;

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
    renderEmptyList("largest-list", "Analyze a project to populate file distribution.");
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
    list.appendChild(emptyMessage("No files matched the scanner rules."));
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
    meta.textContent = largestMode === "size" ? entry.sizeHuman : formatCount(value);

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
      ? "Compact Prompt"
      : analysis.estimatedTokens < 120000
        ? "Review-sized Prompt"
        : "Large Context";

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
  const insights = [`${formatCount(analysis.estimatedTokens)} estimated tokens across ${formatCount(analysis.processedFiles)} files.`];

  if (options.maxOutputLines) {
    insights.push(`Split output is active at ${formatCount(options.maxOutputLines)} lines per report part.`);
  } else if (analysis.totalLines > SPLIT_SAFE_LINES) {
    insights.push(`Set split output near ${formatCount(SPLIT_SAFE_LINES)} lines to reduce oversized reports.`);
  } else {
    insights.push("Split output is optional for the current line count.");
  }

  const topFile = analysis.largestByTokens[0];
  if (topFile) {
    insights.push(`${topFile.path} is the largest token contributor at ${formatCount(topFile.estimatedTokens)} tokens.`);
  }

  const topExtension = analysis.extensionBreakdown[0];
  if (topExtension && analysis.estimatedTokens > 0) {
    const share = Math.round((topExtension.estimatedTokens / analysis.estimatedTokens) * 100);
    insights.push(`${topExtension.extension} files account for ${share}% of estimated tokens.`);
  }

  if (options.noGitignore) {
    insights.push(".gitignore is disabled, so generated or vendored files may enter the scan.");
  }

  return insights;
}

function renderDiagnostics(analysis: ProjectAnalysis): void {
  byId("config-state").textContent = analysis.config.configPresent ? "Project config" : "Defaults";
  byId("gitignore-state").textContent = analysis.config.gitignoreEnabled ? "Enabled" : "Disabled";
  byId("max-size-state").textContent = analysis.config.maxFileSizeHuman;

  const list = byId("skipped-list");
  list.replaceChildren();
  if (analysis.skippedReasons.length === 0) {
    list.appendChild(emptyMessage("No skipped files."));
    return;
  }

  const max = Math.max(...analysis.skippedReasons.map((entry) => entry.files), 1);
  analysis.skippedReasons.forEach((entry) => {
    const li = document.createElement("li");
    const label = document.createElement("span");
    label.textContent = entry.reason;
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
    list.appendChild(emptyMessage("No saved reports yet."));
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
    reveal.title = "Reveal in Finder";
    reveal.innerHTML = folderIcon();
    reveal.addEventListener("click", () => {
      revealItemInDir(path).catch((err) => showError(String(err)));
    });

    const copy = document.createElement("button");
    copy.type = "button";
    copy.className = "icon-btn mini";
    copy.title = "Copy path";
    copy.innerHTML = copyIcon();
    copy.addEventListener("click", () => {
      navigator.clipboard
        .writeText(path)
        .then(() => setStatus("Report path copied"))
        .catch(() => showError("Could not copy the report path."));
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

function formatCompact(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${Math.round(value / 1_000)}K`;
  return String(value);
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
    option.textContent = "No recent projects";
    select.appendChild(option);
  } else {
    const placeholder = document.createElement("option");
    placeholder.value = "";
    placeholder.textContent = "Choose recent project";
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
    list.appendChild(emptyMessage("No local history yet."));
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
    const tokens = record.summary ? `${formatCompact(record.summary.estimatedTokens)} tokens` : "not analyzed";
    meta.textContent = `${record.projectType || "Generic"} - ${tokens}`;
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
  setStatus("Recent project loaded");
}

async function handleAnalyze(): Promise<void> {
  const path = input("project-input").value.trim();
  if (!path) {
    showError("Choose a project folder before analyzing.");
    return;
  }

  hideError();
  setLoading("analyze", true);
  setStatus("Analyzing project");

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
    setStatus(`Analyzed ${formatCount(analysis.processedFiles)} files`);
  } catch (err) {
    showError(typeof err === "string" ? err : "Could not analyze this project.");
    setStatus("Analysis failed");
  } finally {
    setLoading("analyze", false);
  }
}

async function handleRunScan(): Promise<void> {
  const inputDir = input("project-input").value.trim();
  if (!inputDir) {
    showError("Choose a project folder before running a scan.");
    return;
  }

  const outputInput = input("output-input");
  if (!outputInput.value.trim()) {
    outputInput.value = suggestOutputDir(inputDir);
  }

  hideError();
  setLoading("scan", true);
  setStatus("Running scan");

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
    setStatus(`Saved ${outcome.outputPaths.length.toLocaleString()} report file${outcome.outputPaths.length === 1 ? "" : "s"}`);
  } catch (err) {
    showError(typeof err === "string" ? err : "Could not run the scan.");
    setStatus("Scan failed");
  } finally {
    setLoading("scan", false);
  }
}

function bindEvents(): void {
  byId<HTMLButtonElement>("browse-project").addEventListener("click", async () => {
    const folder = await chooseFolder();
    if (!folder) return;
    setProjectPath(folder);
    setStatus("Project selected");
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
      setStatus(`${button.textContent?.trim() ?? "Preset"} preset applied`);
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
  renderEmptyComposition("No analysis yet");
  renderEmptyList("largest-list", "Analyze a project to populate file distribution.");
  renderEmptyList("readiness-list", "Analyze a project to calculate prompt readiness.");
  renderEmptyList("skipped-list", "Analyze a project to inspect skipped files.");
  renderReports([]);
  historyRecords = loadHistory();
  renderHistory();
}

function main(): void {
  initTheme();
  initTitlebarDrag();
  renderInitialState();
  bindEvents();
}

main();
