import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import "./style.css";

type Mode = "scan" | "loc";

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

interface LocFileEntry {
  path: string;
  lines: number;
  sizeHuman: string;
}

interface LocStats {
  processedFiles: number;
  skippedFiles: number;
  totalLines: number;
  totalChars: number;
  estimatedTokens: number;
  largestFiles: LocFileEntry[];
}

const THEME_KEY = "code-scanner-theme";

function initTheme(): void {
  const saved = localStorage.getItem(THEME_KEY);
  if (saved === "light") {
    document.documentElement.dataset.theme = "light";
  }
  document.getElementById("theme-toggle")?.addEventListener("click", () => {
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

function initModeSwitch(onChange: (mode: Mode) => void): void {
  const tabs = Array.from(document.querySelectorAll<HTMLButtonElement>(".mode-tab"));
  const glider = document.querySelector<HTMLDivElement>(".mode-tab-glider");

  const positionGlider = (tab: HTMLButtonElement) => {
    if (!glider) return;
    glider.style.width = `${tab.offsetWidth}px`;
    glider.style.transform = `translateX(${tab.offsetLeft - 4}px)`;
  };

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      tabs.forEach((t) => {
        t.classList.toggle("is-active", t === tab);
        t.setAttribute("aria-selected", String(t === tab));
      });
      positionGlider(tab);
      const mode = tab.dataset.mode as Mode;
      document.querySelectorAll<HTMLElement>(".panel").forEach((panel) => {
        panel.hidden = panel.dataset.panel !== mode;
      });
      onChange(mode);
    });
  });

  const active = tabs.find((t) => t.classList.contains("is-active")) ?? tabs[0];
  requestAnimationFrame(() => positionGlider(active));
  window.addEventListener("resize", () => {
    const current = tabs.find((t) => t.classList.contains("is-active")) ?? tabs[0];
    positionGlider(current);
  });
}

function parseIgnoreList(raw: string): string[] {
  return raw
    .split(/[\s,]+/)
    .map((s) => s.trim().replace(/^\./, ""))
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

function initBrowseButtons(): void {
  document.querySelectorAll<HTMLButtonElement>("[data-browse]").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const targetId = btn.dataset.browse;
      if (!targetId) return;
      const input = document.getElementById(targetId) as HTMLInputElement | null;
      if (!input) return;

      const selection = await open({ directory: true, multiple: false });
      if (typeof selection !== "string") return;

      input.value = selection;

      if (targetId === "scan-input") {
        const output = document.getElementById("scan-output") as HTMLInputElement | null;
        if (output && !output.value) {
          output.value = suggestOutputDir(selection);
        }
      }
    });
  });
}

function setLoading(button: HTMLButtonElement, loading: boolean): void {
  button.disabled = loading;
  button.classList.toggle("is-loading", loading);
}

function showError(message: string): void {
  const banner = document.getElementById("error-banner");
  if (!banner) return;
  banner.textContent = message;
  banner.hidden = false;
}

function hideError(): void {
  const banner = document.getElementById("error-banner");
  if (banner) banner.hidden = true;
}

function showEmptyState(show: boolean): void {
  const empty = document.getElementById("empty-state");
  if (empty) empty.hidden = !show;
}

function renderScanResult(outcome: ScanOutcome): void {
  document.getElementById("rs-project-name")!.textContent = outcome.projectName;
  document.getElementById("rs-project-type")!.textContent = outcome.projectType;
  document.getElementById("rs-processed")!.textContent = outcome.processedFiles.toLocaleString();
  document.getElementById("rs-skipped")!.textContent = outcome.skippedFiles.toLocaleString();
  document.getElementById("rs-size")!.textContent = outcome.totalSizeHuman;

  const list = document.getElementById("rs-paths")!;
  list.innerHTML = "";
  outcome.outputPaths.forEach((path) => {
    const li = document.createElement("li");

    const span = document.createElement("span");
    span.className = "path-text";
    span.textContent = path;
    span.title = path;

    const reveal = document.createElement("button");
    reveal.type = "button";
    reveal.className = "reveal-btn";
    reveal.title = "Reveal in Finder";
    reveal.innerHTML =
      '<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 4h6l2 2h8v12H4V4Z" stroke-linejoin="round"/></svg>';
    reveal.addEventListener("click", () => {
      revealItemInDir(path).catch((err) => showError(String(err)));
    });

    li.append(span, reveal);
    list.appendChild(li);
  });
}

function renderLocResult(stats: LocStats): void {
  document.getElementById("loc-processed")!.textContent = stats.processedFiles.toLocaleString();
  document.getElementById("loc-skipped")!.textContent = stats.skippedFiles.toLocaleString();
  document.getElementById("loc-lines")!.textContent = stats.totalLines.toLocaleString();
  document.getElementById("loc-chars")!.textContent = stats.totalChars.toLocaleString();
  document.getElementById("loc-tokens")!.textContent = stats.estimatedTokens.toLocaleString();

  const list = document.getElementById("loc-top-files")!;
  list.innerHTML = "";
  const maxLines = stats.largestFiles[0]?.lines || 1;

  stats.largestFiles.forEach((entry, index) => {
    const li = document.createElement("li");

    const rank = document.createElement("span");
    rank.className = "rank";
    rank.textContent = String(index + 1);

    const main = document.createElement("div");
    main.className = "top-file-main";

    const pathEl = document.createElement("span");
    pathEl.className = "top-file-path";
    pathEl.textContent = entry.path;
    pathEl.title = entry.path;

    const track = document.createElement("div");
    track.className = "bar-track";
    const fill = document.createElement("div");
    fill.className = "bar-fill";
    track.appendChild(fill);

    main.append(pathEl, track);

    const meta = document.createElement("span");
    meta.className = "top-file-meta";
    meta.textContent = `${entry.lines.toLocaleString()} lines · ${entry.sizeHuman}`;

    li.append(rank, main, meta);
    list.appendChild(li);

    requestAnimationFrame(() => {
      fill.style.width = `${Math.max(4, (entry.lines / maxLines) * 100)}%`;
    });
  });
}

function setResultsVisible(mode: Mode | null): void {
  const scanResults = document.getElementById("scan-results")!;
  const locResults = document.getElementById("loc-results")!;
  scanResults.hidden = mode !== "scan";
  locResults.hidden = mode !== "loc";
  showEmptyState(mode === null);
}

function initScanForm(onResult: (mode: Mode) => void): void {
  const form = document.getElementById("scan-form") as HTMLFormElement;
  const runBtn = document.getElementById("scan-run") as HTMLButtonElement;

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    hideError();

    const inputDir = (document.getElementById("scan-input") as HTMLInputElement).value.trim();
    const outputDir = (document.getElementById("scan-output") as HTMLInputElement).value.trim();
    const verbose = (document.getElementById("scan-verbose") as HTMLInputElement).checked;
    const noGitignore = (document.getElementById("scan-no-gitignore") as HTMLInputElement).checked;
    const ignore = parseIgnoreList((document.getElementById("scan-ignore") as HTMLInputElement).value);
    const maxLinesRaw = (document.getElementById("scan-max-lines") as HTMLInputElement).value;
    const maxOutputLines = maxLinesRaw ? Number(maxLinesRaw) : null;

    const options: ScanOptionsPayload = { noGitignore, verbose, ignore, maxOutputLines };

    setLoading(runBtn, true);
    try {
      const outcome = await invoke<ScanOutcome>("run_scan", { inputDir, outputDir, options });
      renderScanResult(outcome);
      setResultsVisible("scan");
      onResult("scan");
    } catch (err) {
      showError(typeof err === "string" ? err : "Something went wrong while scanning.");
      setResultsVisible(null);
    } finally {
      setLoading(runBtn, false);
    }
  });
}

function initLocForm(onResult: (mode: Mode) => void): void {
  const form = document.getElementById("loc-form") as HTMLFormElement;
  const runBtn = document.getElementById("loc-run") as HTMLButtonElement;

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    hideError();

    const path = (document.getElementById("loc-input") as HTMLInputElement).value.trim();
    const noGitignore = (document.getElementById("loc-no-gitignore") as HTMLInputElement).checked;
    const ignore = parseIgnoreList((document.getElementById("loc-ignore") as HTMLInputElement).value);

    const options: ScanOptionsPayload = { noGitignore, verbose: false, ignore, maxOutputLines: null };

    setLoading(runBtn, true);
    try {
      const stats = await invoke<LocStats>("run_loc", { path, options });
      renderLocResult(stats);
      setResultsVisible("loc");
      onResult("loc");
    } catch (err) {
      showError(typeof err === "string" ? err : "Something went wrong while analyzing.");
      setResultsVisible(null);
    } finally {
      setLoading(runBtn, false);
    }
  });
}

function main(): void {
  initTheme();
  initBrowseButtons();

  let lastResultMode: Mode | null = null;
  initModeSwitch((mode) => {
    hideError();
    setResultsVisible(lastResultMode === mode ? mode : null);
  });
  initScanForm((mode) => {
    lastResultMode = mode;
  });
  initLocForm((mode) => {
    lastResultMode = mode;
  });
}

main();
