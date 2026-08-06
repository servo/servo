// Helpers for the modulepreload tests that drive resources/preload-module.py.
// Paths are resolved against the test page, which is expected to be in
// preload/.

const MODULE_HANDLER = "./resources/preload-module.py";

function moduleUrl(params) {
  const url = new URL(MODULE_HANDLER, location.href);
  for (const [key, value] of Object.entries(params)) {
    url.searchParams.set(key, value);
  }
  return url.href;
}

// NOTE: crossorigin is what makes these tests exercise the "already in the
// module map" path in implementations that key their preload cache on the CORS
// mode: the preload and the import() of the same URL then don't share a preload
// entry, so the preload has to be satisfied from the module map. Keep it.
function makeModulePreload(url) {
  const link = document.createElement("link");
  link.rel = "modulepreload";
  link.as = "script";
  link.crossOrigin = "";
  link.href = url;
  return link;
}

function eventFor(element) {
  return new Promise(resolve => {
    element.addEventListener("load", () => resolve("load"), { once: true });
    element.addEventListener("error", () => resolve("error"), { once: true });
  });
}

// Lets a request blocked by preload-module.py's block=1 finish.
function release(uuid) {
  return fetch(moduleUrl({ release: 1, uuid }));
}

// Whether the response for `url` has been received. Resource timing is what the
// tests use to observe the server's progress, rather than asking the server, so
// that nothing has to keep mutable state that concurrent requests could race
// over.
function hasResourceLoaded(url) {
  return performance.getEntriesByName(url).length > 0;
}

function resourceLoaded(url) {
  return new Promise(resolve => {
    if (hasResourceLoaded(url)) {
      resolve();
      return;
    }
    const observer = new PerformanceObserver(entries => {
      if (entries.getEntries().some(entry => entry.name === url)) {
        observer.disconnect();
        resolve();
      }
    });
    observer.observe({ type: "resource", buffered: true });
  });
}
