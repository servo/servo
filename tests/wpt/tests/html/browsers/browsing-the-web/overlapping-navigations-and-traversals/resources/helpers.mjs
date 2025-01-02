export function createIframe(t) {
  return new Promise((resolve, reject) => {
    const iframe = document.createElement("iframe");
    iframe.onload = () => resolve(iframe);
    iframe.onerror = () => reject(new Error("Could not load iframe"));
    iframe.src = "/common/blank.html";

    t.add_cleanup(() => iframe.remove());
    document.body.append(iframe);
  });
}

export function delay(t, ms) {
  return new Promise(resolve => t.step_timeout(resolve, ms));
}

export function waitForLoad(obj) {
  return new Promise(resolve => {
    obj.addEventListener("load", resolve, { once: true });
  });
}

export function waitForHashchange(obj) {
  return new Promise(resolve => {
    obj.addEventListener("hashchange", resolve, { once: true });
  });
}

export function waitForPopstate(obj) {
  return new Promise(resolve => {
    obj.addEventListener("popstate", resolve, { once: true });
  });
}

// This is used when we want to end the test by asserting some load doesn't
// happen, but we're not sure how long to wait. We could just wait a long-ish
// time (e.g. a second), but that makes the tests slow. Instead, assume that
// network loads take roughly the same time. Then, you can use this function to
// wait a small multiple of the duration of a separate iframe load; this should
// be long enough to catch any problems.
export async function waitForPotentialNetworkLoads(t) {
  const before = performance.now();

  // Sometimes we're doing something, like a traversal, which cancels our first
  // attempt at iframe loading. In that case we bail out after 100 ms and try
  // again. (Better ideas welcome...)
  await Promise.race([createIframe(t), delay(t, 100)]);
  await createIframe(t);

  const after = performance.now();
  await delay(t, after - before);
}
