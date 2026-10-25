// Opts in to cross-document view transitions. While the test arms it, the
// first pageswap handler adds a session history entry with pushState.
const style = document.createElement("style");
style.textContent = "@view-transition { navigation: auto; }";
document.head.append(style);

const PREFIX = "pageswap-history-change-during-traverse-";

addEventListener("pageswap", e => {
  if (localStorage.getItem(PREFIX + "armed") !== "1") {
    return;
  }
  const events = JSON.parse(localStorage.getItem(PREFIX + "events") || "[]");
  events.push({ navigationType: e.activation?.navigationType ?? null });
  localStorage.setItem(PREFIX + "events", JSON.stringify(events));
  if (events.length == 1) {
    history.pushState(null, "", "#pushed");
  }
});
