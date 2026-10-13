import "@fontsource/barlow-condensed/600.css";
import "@fontsource/barlow-condensed/700.css";
import "@fontsource/ibm-plex-sans/400.css";
import "@fontsource/ibm-plex-sans/500.css";
import "@fontsource/ibm-plex-sans/600.css";
import type {
  Catalog,
  ChangeRequest,
  Entry,
  EntryKind,
  Preview,
  ProfileSnapshot,
} from "./types";
import "./style.css";
import type { DownloadResult, ProfileBackend } from "./backend";

const MAX_COINS = 2147483647;
const web = import.meta.env.MODE === "web";
const desktop = !web && "__TAURI_INTERNALS__" in window;
let backend: ProfileBackend | null = null;
const kinds: { id: EntryKind; label: string; stamp: string }[] = [
  { id: "characters", label: "Survivors", stamp: "SV" },
  { id: "skills", label: "Skills", stamp: "SK" },
  { id: "achievements", label: "Achievements", stamp: "AC" },
  { id: "skins", label: "Skins", stamp: "SN" },
  { id: "artifacts", label: "Artifacts", stamp: "AR" },
  { id: "items", label: "Items", stamp: "IT" },
  { id: "equipment", label: "Equipment", stamp: "EQ" },
];
const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("Application root is missing.");
// Only fixed application markup belongs here. All external values use textContent.
app.innerHTML = `
  <header class="masthead">
    <div class="brand-mark" aria-hidden="true">RoR<span>2</span></div>
    <div><p class="eyebrow">Survivor field terminal</p><h1>Profile unlocker</h1></div>
    <span class="runtime-tag" id="runtime-tag"></span>
  </header>
  <main>
    <section class="profile-bar" aria-labelledby="profile-title">
      <div class="profile-heading"><p class="eyebrow">Active profile</p><h2 id="profile-title">No profile connected</h2><p id="profile-path" class="mono subdued">Choose a local UserProfile XML file.</p></div>
      <fieldset id="profile-controls" class="profile-controls"><legend class="sr-only">Open a profile</legend>
        <button id="pick-profile" class="primary" type="button">Open profile XML</button>
        <form id="path-form"><label for="manual-path">Or enter a file path</label><div class="input-action"><input id="manual-path" type="text" placeholder="Full path to your profile.xml" spellcheck="false" autocomplete="off" required /><button type="submit" id="open-path">Open path</button></div></form>
      </fieldset>
      <input id="profile-file" type="file" accept=".xml" aria-label="Choose profile XML" hidden />
    </section>
    <div id="notice" class="notice" role="status" aria-live="polite" aria-atomic="true"></div>
    <div id="error" class="error" role="alert" hidden></div>
    <button id="retry-catalog" type="button" hidden>Retry loading catalog</button>
    <section id="browser-guidance" class="desktop-guidance" hidden><h2>Browser edition · local files only</h2><p>Your XML stays in this browser tab. It is not uploaded or stored. Prepare and download the modified profile and exact original backup, then manually replace the original with the game closed. This page cannot write to or replace your source file.</p></section>
    <section id="desktop-guidance" class="desktop-guidance" hidden><h2>Run the desktop app to open profiles</h2><p>This build requires the Linux or Windows desktop application. Use the browser edition to edit a selected XML file locally and download the result.</p></section>
    <div class="workspace">
      <nav class="category-rail" aria-label="Unlock categories"><p class="eyebrow">Manifest sections</p><div id="categories"></div><div class="rail-note"><span class="signal-dot" aria-hidden="true"></span><p>Additive edits only.<br />Existing progress stays intact.</p></div></nav>
      <section class="manifest" aria-labelledby="manifest-title">
        <div class="section-heading"><div><p class="eyebrow">Unlock manifest</p><h2 id="manifest-title">All entries</h2></div><span id="manifest-count" class="mono subdued"></span></div>
        <fieldset id="manifest-controls" class="manifest-controls"><legend class="sr-only">Filter and stage entries</legend>
          <label class="search-label" for="search">Search the manifest<input id="search" type="search" placeholder="Name, identifier, or description" /></label>
          <div class="filters"><label for="expansion">Expansion<select id="expansion"><option value="">All expansions</option></select></label><label class="checkbox-label"><input id="locked-only" type="checkbox" /> Missing only</label></div>
          <div class="bulk-actions"><button id="select-visible" type="button">Stage visible missing</button><button id="select-all" type="button">Stage all missing</button></div>
        </fieldset>
        <div class="manifest-key" aria-hidden="true"><span>Entry / internal identifier</span><span>Profile state</span></div>
        <div id="entries" class="entries"></div>
        <p id="catalog-source" class="catalog-source subdued"></p>
      </section>
      <aside class="inspector" aria-labelledby="inspector-title">
        <div class="section-heading"><div><p class="eyebrow">Pending changes</p><h2 id="inspector-title">Staging area</h2></div><span id="staged-count" class="count-badge">0</span></div>
        <fieldset id="staging-controls"><legend class="sr-only">Staged unlocks and lunar coins</legend>
          <div class="staged-heading"><span class="subdued">Selected entries</span><button id="clear-selection" type="button" class="text-button">Clear</button></div>
          <ul id="staged-list" class="staged-list"></ul>
          <section class="coins-section" aria-labelledby="coins-title"><h3 id="coins-title">Lunar coins</h3><p class="subdued">Current balance: <strong id="coin-balance">—</strong></p><label for="coins">New balance <span class="subdued">(optional)</span></label><div class="input-action"><input id="coins" type="text" inputmode="numeric" placeholder="Leave unchanged" aria-describedby="coin-help coin-error" /><button id="max-coins" type="button">Max</button></div><p id="coin-error" class="validation" aria-live="polite"></p><p id="coin-help" class="fine-print">Whole numbers from 0 to 2,147,483,647. The maximum may overflow game-side calculations; a smaller balance is safer.</p></section>
        </fieldset>
        <section class="preview-section" aria-labelledby="preview-title"><h3 id="preview-title">Exact preview</h3><div id="preview-content"><p class="subdued">Stage changes, then preview the fields that will be added.</p></div></section>
        <div class="save-actions"><button id="preview-button" class="primary" type="button">Preview changes</button><button id="save-button" type="button">Save with backup</button></div>
        <section id="downloads" class="backup-result" aria-label="Prepared profile downloads" hidden><h3>Downloads ready</h3><p>Download both files, then replace the original with the game closed. Your selected original is unchanged.</p><div class="download-actions"><a id="download-modified">Download modified profile</a><a id="download-backup">Download original backup</a></div></section>
        <div class="safety-note"><h3 id="safety-title">Before saving</h3><p>Close Risk of Rain 2. Steam Cloud can replace local edits; let synchronization finish and resolve any conflict carefully.</p><p id="save-guidance">Each changed save creates a unique backup beside the original. DLC ownership and Steam achievements are not changed.</p></div>
      </aside>
    </div>
  </main>
  <footer><span>Risk of Rain 2 / local profile operations</span><span id="workflow-label">Preview → backup → save</span></footer>
  <dialog id="discard-dialog" aria-labelledby="discard-title"><form method="dialog"><p class="eyebrow">Unsaved changes</p><h2 id="discard-title">Open another profile?</h2><p>Your staged unlocks and coin edit will be discarded after the new profile opens successfully.</p><div class="dialog-actions"><button value="cancel" autofocus>Keep editing</button><button value="discard" class="primary">Open another profile</button></div></form></dialog>
`;

function element<T extends HTMLElement = HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`Missing interface element: ${id}`);
  return found as T;
}
function node<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  text = "",
  className = "",
): HTMLElementTagNameMap[K] {
  const result = document.createElement(tag);
  result.textContent = text;
  if (className) result.className = className;
  return result;
}

let catalog: Catalog | null = null;
let profile: ProfileSnapshot | null = null;
let unlocked = new Set<string>();
const selection = new Set<string>();
let category = "";
let busy = false;
let preview: Preview | null = null;
let previewRequest: ChangeRequest | null = null;
let lastBackup: string | null = null;
let prepared: { modified: string; backup: string } | null = null;
const search = element<HTMLInputElement>("search");
const expansion = element<HTMLSelectElement>("expansion");
const lockedOnly = element<HTMLInputElement>("locked-only");
const coinsInput = element<HTMLInputElement>("coins");

function status(message: string): void {
  element("notice").textContent = message;
}
function coinValue(): number | null {
  const text = coinsInput.value.trim();
  if (!text) return null;
  if (!/^\d+$/.test(text))
    throw new Error("Enter a whole number without decimals or separators.");
  const value = Number(text);
  if (!Number.isSafeInteger(value) || value > MAX_COINS)
    throw new Error("Coin balance must be between 0 and 2,147,483,647.");
  return value;
}
function hasStagedChanges(): boolean {
  if (selection.size) return true;
  if (!profile || !coinsInput.value.trim()) return false;
  try {
    return coinValue() !== profile.coins;
  } catch {
    return true;
  }
}
function request(): ChangeRequest {
  if (!profile) throw new Error("Open a profile first.");
  return {
    path: profile.path,
    revision: profile.revision,
    selection: [...selection].sort(),
    coins: coinValue(),
  };
}
function disposeDownloads(): void {
  if (prepared) {
    URL.revokeObjectURL(prepared.modified);
    URL.revokeObjectURL(prepared.backup);
    prepared = null;
  }
  for (const id of ["download-modified", "download-backup"]) {
    const link = element<HTMLAnchorElement>(id);
    link.removeAttribute("href");
    link.removeAttribute("download");
  }
  element("downloads").hidden = true;
}

function prepareDownloads(result: DownloadResult): void {
  disposeDownloads();
  if (
    !result.preview.total_additions &&
    result.preview.coins_before === result.preview.coins_after
  )
    return;
  const modified = URL.createObjectURL(
    new Blob([result.xml], {
      type: "application/xml;charset=utf-8",
      endings: "transparent",
    }),
  );
  let backup: string;
  try {
    backup = URL.createObjectURL(
      new Blob([result.original], { type: "application/xml;charset=utf-8" }),
    );
  } catch (error) {
    URL.revokeObjectURL(modified);
    throw error;
  }
  prepared = { modified, backup };
  const modifiedLink = element<HTMLAnchorElement>("download-modified");
  modifiedLink.href = modified;
  modifiedLink.download = `${result.filename.replace(/\.xml$/i, "")}.unlocked.xml`;
  const backupLink = element<HTMLAnchorElement>("download-backup");
  backupLink.href = backup;
  backupLink.download = `${result.filename}.backup.xml`;
  element("downloads").hidden = false;
}
function invalidatePreview(): void {
  disposeDownloads();
  preview = null;
  previewRequest = null;
  lastBackup = null;
  element("error").hidden = true;
  status(
    web
      ? "Staged changes updated. Preview again before preparing downloads."
      : "Staged changes updated. Preview again before saving.",
  );
}
function visibleEntries(): Entry[] {
  if (!catalog) return [];
  const query = search.value.trim().toLocaleLowerCase();
  return catalog.entries.filter(
    (entry) =>
      (!category || entry.kind === category) &&
      (!expansion.value || entry.expansion === expansion.value) &&
      (!lockedOnly.checked || !unlocked.has(entry.id)) &&
      (!query ||
        `${entry.label} ${entry.id} ${entry.description}`
          .toLocaleLowerCase()
          .includes(query)),
  );
}

function renderCategories(): void {
  const container = element("categories");
  container.replaceChildren();
  const categories = [{ id: "", label: "All entries", stamp: "ALL" }, ...kinds];
  for (const item of categories) {
    const entries =
      catalog?.entries.filter((entry) => !item.id || entry.kind === item.id) ??
      [];
    const completed = entries.filter((entry) => unlocked.has(entry.id)).length;
    const button = node(
      "button",
      "",
      `category-button ${category === item.id ? "active" : ""}`,
    );
    button.type = "button";
    button.disabled = busy || !catalog;
    button.setAttribute("aria-pressed", String(category === item.id));
    button.append(
      node("span", item.stamp, `category-stamp ${item.id}`),
      node("span", item.label, "category-name"),
      node("span", `${completed}/${entries.length}`, "category-count mono"),
    );
    button.title = `${completed} complete of ${entries.length} entries`;
    button.addEventListener("click", () => {
      category = item.id;
      renderCategories();
      renderEntries();
      container
        .querySelector<HTMLButtonElement>('button[aria-pressed="true"]')
        ?.focus();
    });
    container.append(button);
  }
}
function renderEntries(): void {
  const container = element("entries");
  const entries = visibleEntries();
  container.replaceChildren();
  element("manifest-title").textContent =
    kinds.find((kind) => kind.id === category)?.label ?? "All entries";
  element("manifest-count").textContent = `${entries.length} entries`;
  if (!catalog || !profile) {
    const empty = node("div", "", "empty-state");
    empty.append(
      node(
        "h3",
        catalog
          ? "Connect your survivor profile"
          : "Loading the unlock catalog",
      ),
    );
    empty.append(
      node(
        "p",
        catalog
          ? "Open a UserProfile XML to see your progress and stage missing entries."
          : "The catalog is loaded from the desktop app.",
      ),
    );
    empty.append(
      node(
        "p",
        "Unlock survivors, skills, skins, artifacts, and achievement rewards. Unlock items and equipment and mark them found, or set your lunar coin balance.",
        "subdued",
      ),
    );
    empty.append(
      node(
        "p",
        "Nothing is written until you preview and save. A changed save keeps a backup of the original XML.",
        "fine-print",
      ),
    );
    if (!catalog && desktop && !busy) {
      const retry = node("button", "Retry loading catalog");
      retry.type = "button";
      retry.addEventListener("click", () => {
        void loadCatalog();
      });
      empty.append(retry);
    }
    container.append(empty);
  } else if (!entries.length) {
    container.append(
      node(
        "div",
        "No entries match these filters. Try another search, category, or expansion.",
        "empty-state subdued",
      ),
    );
  } else {
    for (const entry of entries) {
      const complete = unlocked.has(entry.id);
      const staged = selection.has(entry.id);
      const row = node(
        "label",
        "",
        `manifest-row ${complete ? "complete" : staged ? "staged" : "locked"}`,
      );
      const checkbox = node("input");
      checkbox.type = "checkbox";
      checkbox.id = `entry-${entry.id}`;
      checkbox.checked = complete || staged;
      checkbox.disabled = busy || complete;
      checkbox.setAttribute(
        "aria-label",
        `${complete ? "Complete" : "Stage"}: ${entry.label}`,
      );
      checkbox.addEventListener("change", () => toggleSelection(entry.id));
      const info = node("div", "", "entry-info");
      const heading = node("div", "", "entry-heading");
      heading.append(
        node("span", entry.label, "entry-label"),
        node("span", entry.expansion || "Base game", "expansion-tag"),
      );
      info.append(heading, node("span", entry.id, "entry-id mono"));
      if (entry.description)
        info.append(node("p", entry.description, "entry-description"));
      row.append(
        node(
          "span",
          kinds.find((kind) => kind.id === entry.kind)?.stamp ?? "—",
          `row-stamp ${entry.kind}`,
        ),
        checkbox,
        info,
        node(
          "span",
          complete ? "Complete" : staged ? "Staged" : "Missing",
          "entry-status",
        ),
      );
      container.append(row);
    }
  }
  element<HTMLButtonElement>("select-visible").disabled =
    busy ||
    !profile ||
    !entries.some(
      (entry) => !unlocked.has(entry.id) && !selection.has(entry.id),
    );
  element<HTMLButtonElement>("select-all").disabled =
    busy ||
    !profile ||
    !catalog?.entries.some(
      (entry) => !unlocked.has(entry.id) && !selection.has(entry.id),
    );
}
function toggleSelection(id: string): void {
  if (busy || !profile || unlocked.has(id)) return;
  const focusedEntry = document.activeElement?.id === `entry-${id}`;
  if (selection.has(id)) selection.delete(id);
  else selection.add(id);
  invalidatePreview();
  renderEntries();
  renderInspector();
  if (focusedEntry) document.getElementById(`entry-${id}`)?.focus();
}
function renderInspector(): void {
  element("staged-count").textContent = String(selection.size);
  element("coin-balance").textContent = profile
    ? profile.coins.toLocaleString("en-US")
    : "—";
  const list = element("staged-list");
  list.replaceChildren();
  const stagedEntries =
    catalog?.entries.filter((entry) => selection.has(entry.id)) ?? [];
  if (!stagedEntries.length)
    list.append(node("li", "No unlocks staged.", "subdued"));
  for (const entry of stagedEntries) {
    const item = node("li");
    const remove = node("button", "×", "remove-entry");
    remove.type = "button";
    remove.disabled = busy;
    remove.setAttribute("aria-label", `Remove ${entry.label} from staging`);
    remove.addEventListener("click", () => toggleSelection(entry.id));
    item.append(node("span", entry.label), remove);
    list.append(item);
  }
  let coinError = "";
  try {
    coinValue();
  } catch (error) {
    coinError = error instanceof Error ? error.message : String(error);
  }
  element("coin-error").textContent = coinError;
  coinsInput.setAttribute("aria-invalid", String(Boolean(coinError)));
  element<HTMLButtonElement>("clear-selection").disabled =
    busy || !selection.size;
  const content = element("preview-content");
  content.replaceChildren();
  if (preview) {
    const coinChanged = preview.coins_after !== preview.coins_before;
    content.append(
      node(
        "p",
        `${preview.total_additions} field additions${coinChanged ? " · coin balance changes" : " · coins unchanged"}`,
        "preview-summary",
      ),
    );
    if (!preview.total_additions && !coinChanged)
      content.append(
        node(
          "p",
          web
            ? "No changes needed. No modified download or backup will be prepared."
            : "No changes needed. Saving this preview will not write the file or create a backup.",
          "subdued",
        ),
      );
    if (coinChanged)
      content.append(
        node(
          "p",
          `Lunar coins: ${preview.coins_before.toLocaleString("en-US")} → ${preview.coins_after.toLocaleString("en-US")}`,
          "mono",
        ),
      );
    for (const change of preview.changes) {
      if (!change.added.length) continue;
      const details = node("details", "", "field-preview");
      details.append(
        node("summary", `${change.field} +${change.added.length}`),
      );
      const additions = node("ul", "", "mono");
      for (const addition of change.added)
        additions.append(node("li", addition));
      details.append(additions);
      content.append(details);
    }
  } else
    content.append(
      node(
        "p",
        "Stage changes, then preview the exact additions. Changing staged entries or coins clears the preview.",
        "subdued",
      ),
    );
  if (lastBackup) {
    const backup = node("div", "", "backup-result");
    backup.append(
      node("strong", "Backup saved"),
      node("p", lastBackup, "mono"),
    );
    content.append(backup);
  }
  element<HTMLButtonElement>("preview-button").disabled =
    busy || !profile || !catalog || Boolean(coinError);
  element<HTMLButtonElement>("save-button").disabled =
    busy || !preview || !previewRequest || Boolean(coinError);
}
function render(): void {
  element<HTMLFieldSetElement>("profile-controls").disabled =
    busy || !backend || !catalog;
  element<HTMLButtonElement>("retry-catalog").disabled = busy;
  element("retry-catalog").hidden = Boolean(catalog) || (!web && !desktop);
  for (const id of ["download-modified", "download-backup"]) {
    const link = element<HTMLAnchorElement>(id);
    link.setAttribute("aria-disabled", String(busy));
    link.tabIndex = busy ? -1 : 0;
  }
  element<HTMLFieldSetElement>("manifest-controls").disabled = busy || !catalog;
  element<HTMLFieldSetElement>("staging-controls").disabled = busy || !profile;
  app?.setAttribute("aria-busy", String(busy));
  element("profile-title").textContent =
    profile?.name || "No profile connected";
  element("profile-path").textContent =
    profile?.path || "Choose a local UserProfile XML file.";
  renderCategories();
  renderEntries();
  renderInspector();
}
async function run(
  operation: () => Promise<void>,
  pendingMessage: string,
): Promise<void> {
  if (busy) return;
  const focused = document.activeElement;
  busy = true;
  element("error").hidden = true;
  status(pendingMessage);
  render();
  try {
    await operation();
  } catch (error) {
    const target = element("error");
    target.textContent = error instanceof Error ? error.message : String(error);
    target.hidden = false;
    status(
      web
        ? "Operation failed. Review the error and retry. Your selected original and staged changes are unchanged."
        : "Operation failed. Review the error and retry; re-open the profile if its file changed.",
    );
  } finally {
    busy = false;
    render();
    if (
      focused instanceof HTMLElement &&
      focused.isConnected &&
      !focused.matches(":disabled")
    ) {
      focused.focus({ preventScroll: true });
    }
  }
}
async function confirmProfileChange(): Promise<boolean> {
  if (!hasStagedChanges()) return true;
  const dialog = element<HTMLDialogElement>("discard-dialog");
  // Packaged WebKitGTK targets predate Promise.withResolvers.
  return new Promise<boolean>((resolve) => {
    dialog.returnValue = "cancel";
    dialog.addEventListener(
      "close",
      () => resolve(dialog.returnValue === "discard"),
      { once: true },
    );
    dialog.showModal();
  });
}
function applyProfile(snapshot: ProfileSnapshot): void {
  disposeDownloads();
  profile = snapshot;
  unlocked = new Set(snapshot.unlocked_ids);
  selection.clear();
  coinsInput.value = "";
  preview = null;
  previewRequest = null;
  lastBackup = null;
  element<HTMLInputElement>("manual-path").value = snapshot.path;
}
async function openProfile(path: string): Promise<void> {
  if (!backend?.openPath)
    throw new Error("Manual paths are available only in the desktop app.");
  const snapshot = await backend.openPath(path);
  applyProfile(snapshot);
  status(
    `Opened ${snapshot.name}. Stage missing entries or edit your lunar coin balance.`,
  );
}
async function loadCatalog(): Promise<void> {
  await run(async () => {
    if (!backend) {
      // Platform-specific imports keep generated WASM out of desktop bundles and Tauri out of web bundles.
      if (import.meta.env.MODE === "web")
        backend = (await import("./browser-backend")).backend;
      else if (desktop) backend = (await import("./desktop-backend")).backend;
    }
    if (!backend)
      throw new Error("Open the desktop app or the browser edition.");
    catalog = await backend.getCatalog();
    const allExpansions = node("option", "All expansions");
    allExpansions.value = "";
    expansion.replaceChildren(allExpansions);
    const expansions = [
      ...new Set(catalog.entries.map((entry) => entry.expansion)),
    ].sort();
    for (const value of expansions) {
      if (!value) continue;
      const option = node("option", value);
      option.value = value;
      expansion.append(option);
    }
    element("catalog-source").textContent =
      `Catalog ${catalog.version} · ${catalog.entries.length} entries · ${catalog.source}`;
    status("Catalog ready. Close the game, then open your profile XML.");
  }, "Loading the bundled unlock catalog…");
}

element("pick-profile").addEventListener("click", () => {
  void run(async () => {
    if (!(await confirmProfileChange())) {
      status("Kept the current profile and staged changes.");
      return;
    }
    if (!backend) throw new Error("Wait for the catalog to load.");
    const snapshot = await backend.pickProfile(
      element<HTMLInputElement>("profile-file"),
    );
    if (snapshot) {
      applyProfile(snapshot);
      status(
        `Opened ${snapshot.name}. Stage missing entries or edit your lunar coin balance.`,
      );
    } else
      status("Profile selection canceled. Your staged changes are unchanged.");
  }, "Choosing a profile…");
});
element<HTMLFormElement>("path-form").addEventListener("submit", (event) => {
  event.preventDefault();
  const path = element<HTMLInputElement>("manual-path").value.trim();
  if (!path || !desktop) return;
  void run(async () => {
    if (!(await confirmProfileChange())) {
      status("Kept the current profile and staged changes.");
      return;
    }
    await openProfile(path);
  }, "Opening profile…");
});
element("retry-catalog").addEventListener("click", () => {
  void loadCatalog();
});
for (const id of ["download-modified", "download-backup"]) {
  element(id).addEventListener("click", (event) => {
    if (busy || !prepared) event.preventDefault();
  });
}
search.addEventListener("input", renderEntries);
expansion.addEventListener("change", renderEntries);
lockedOnly.addEventListener("change", renderEntries);
for (const [id, all] of [
  ["select-visible", false],
  ["select-all", true],
] as const) {
  element(id).addEventListener("click", () => {
    if (busy || !profile) return;
    for (const entry of all ? (catalog?.entries ?? []) : visibleEntries())
      if (!unlocked.has(entry.id)) selection.add(entry.id);
    invalidatePreview();
    renderEntries();
    renderInspector();
  });
}
element("clear-selection").addEventListener("click", () => {
  selection.clear();
  invalidatePreview();
  renderEntries();
  renderInspector();
});
coinsInput.addEventListener("input", () => {
  invalidatePreview();
  renderInspector();
});
element("max-coins").addEventListener("click", () => {
  coinsInput.value = String(MAX_COINS);
  invalidatePreview();
  renderInspector();
});
element("preview-button").addEventListener("click", () => {
  void run(
    async () => {
      const args = request();
      preview = null;
      previewRequest = null;
      if (!backend)
        throw new Error("Open the desktop app or the browser edition.");
      const result = await backend.preview(args);
      preview = result;
      previewRequest = args;
      status(
        result.total_additions || result.coins_before !== result.coins_after
          ? web
            ? "Preview ready. Review the exact additions before preparing downloads."
            : "Preview ready. Review the exact additions before saving."
          : "Preview ready: no changes needed.",
      );
    },
    web
      ? "Previewing changes against the selected original…"
      : "Checking profile revision and previewing changes…",
  );
});
element("save-button").addEventListener("click", () => {
  if (!previewRequest || !preview) return;
  const args = previewRequest;
  void run(
    async () => {
      // Request controls are disabled while pending; only a current successful preview can reach this command.
      if (!backend)
        throw new Error("Open the desktop app or the browser edition.");
      if (!web) {
        preview = null;
        previewRequest = null;
      }
      const result = await backend.save(args);
      if (result.kind === "downloads") {
        prepareDownloads(result);
        preview = result.preview;
        status(
          result.preview.total_additions ||
            result.preview.coins_before !== result.preview.coins_after
            ? "Downloads ready. Download both files, then replace the original with the game closed."
            : "No changes needed. Your original is unchanged; no downloads were prepared.",
        );
      } else {
        applyProfile(result.profile);
        lastBackup = result.backup_path;
        preview = result.preview;
        status(
          result.backup_path
            ? "Profile saved. Staging cleared; the original XML is preserved in the backup below."
            : "No changes needed. The profile was not written and no backup was created.",
        );
      }
    },
    web
      ? "Preparing modified profile and exact original backup…"
      : "Saving profile and preserving the original backup…",
  );
});
window.addEventListener("beforeunload", (event) => {
  if (hasStagedChanges()) {
    event.preventDefault();
    event.returnValue = "";
  }
});

window.addEventListener("pagehide", disposeDownloads);
element("runtime-tag").textContent = web
  ? "Browser edition · local WASM"
  : desktop
    ? "Local desktop session"
    : "Desktop build / browser guidance";
element("desktop-guidance").hidden = desktop || web;
element("browser-guidance").hidden = !web;
element("path-form").hidden = !desktop;
if (web) {
  element("save-button").textContent = "Prepare downloads";
  element("safety-title").textContent = "Before replacing your profile";
  element("save-guidance").textContent =
    "Download and keep the exact original backup before replacing your profile. Downloads do not change the selected source. DLC ownership and Steam achievements are not changed.";
  element("workflow-label").textContent =
    "Preview → download both → manually replace";
}
render();
if (desktop || web) void loadCatalog();
else {
  status(
    "Run the desktop app to open profiles. No profile operations are available in the browser.",
  );
  element("entries").replaceChildren(
    node(
      "div",
      "The desktop app loads the bundled catalog and edits local profiles. No simulated profile is used here.",
      "empty-state subdued",
    ),
  );
}
