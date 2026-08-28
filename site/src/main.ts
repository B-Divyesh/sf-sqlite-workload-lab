import "./style.css";

const tabs = Array.from(document.querySelectorAll<HTMLButtonElement>("[role='tab']"));
const panels = Array.from(document.querySelectorAll<HTMLElement>("[role='tabpanel']"));

function selectTab(next: HTMLButtonElement, focus = false) {
  for (const tab of tabs) {
    const selected = tab === next;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
  }
  for (const panel of panels) {
    panel.hidden = panel.dataset.panelContent !== next.dataset.panel;
  }
  if (focus) next.focus();
}

for (const tab of tabs) {
  tab.addEventListener("click", () => selectTab(tab));
  tab.addEventListener("keydown", (event) => {
    const current = tabs.indexOf(tab);
    let next = current;
    if (event.key === "ArrowRight") next = (current + 1) % tabs.length;
    else if (event.key === "ArrowLeft") next = (current - 1 + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else return;
    event.preventDefault();
    selectTab(tabs[next], true);
  });
}

const copyStatus = document.querySelector<HTMLElement>("#copy-status");
for (const button of document.querySelectorAll<HTMLButtonElement>("[data-copy]")) {
  button.addEventListener("click", async () => {
    const content = button.dataset.copy ?? "";
    try {
      await navigator.clipboard.writeText(content);
      const original = button.textContent ?? "Copy";
      button.textContent = "Copied";
      if (copyStatus) copyStatus.textContent = "Command copied to clipboard.";
      window.setTimeout(() => { button.textContent = original; }, 1600);
    } catch {
      if (copyStatus) copyStatus.textContent = "Copy was blocked. Select the command text instead.";
    }
  });
}

const offlineBar = document.querySelector<HTMLElement>("#offline-bar");
function syncConnectionState() {
  if (offlineBar) offlineBar.hidden = navigator.onLine;
}
window.addEventListener("online", syncConnectionState);
window.addEventListener("offline", syncConnectionState);
syncConnectionState();

if ("serviceWorker" in navigator && import.meta.env.PROD) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch(() => {
      // Offline support is progressive; the document remains fully usable.
    });
  });
}
