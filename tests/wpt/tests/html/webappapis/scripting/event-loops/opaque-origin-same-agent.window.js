// https://html.spec.whatwg.org/multipage/webappapis.html#obtain-agent-cluster-key
// https://html.spec.whatwg.org/multipage/browsers.html#determining-the-origin

// A data: URL document has an opaque origin. An about:blank iframe it creates
// inherits that origin, which puts both documents in one similar-origin window
// agent with one event loop.
function runInDataURLDocument(t, steps) {
  const iframe = document.createElement("iframe");
  iframe.src = "data:text/html," + encodeURIComponent(`<body><script>
    const child = document.body.appendChild(document.createElement("iframe"));
    (${steps})(child.contentWindow, result => parent.postMessage(result, "*"));
  <\/script>`);
  const { promise, resolve } = Promise.withResolvers();
  window.addEventListener("message", function listener(event) {
    if (event.source !== iframe.contentWindow)
      return;
    window.removeEventListener("message", listener);
    resolve(event.data);
  });
  document.body.append(iframe);
  t.add_cleanup(() => iframe.remove());
  return promise;
}

promise_test(async t => {
  const log = await runInDataURLDocument(t, (childWindow, report) => {
    const log = [];
    childWindow.queueMicrotask(() => log.push("child"));
    queueMicrotask(() => {
      log.push("parent");
      report(log);
    });
  });
  assert_array_equals(log, ["child", "parent"]);
}, "Microtasks queued in the about:blank child's realm share the parent's microtask queue");

promise_test(async t => {
  const log = await runInDataURLDocument(t, (childWindow, report) => {
    const log = [];
    new MutationObserver(() => log.push("observer")).observe(childWindow.document.body, { childList: true });
    childWindow.document.body.append("text");
    queueMicrotask(() => {
      log.push("microtask");
      report(log);
    });
  });
  assert_array_equals(log, ["observer", "microtask"]);
}, "Mutations in the about:blank child notify observers in the parent's microtask checkpoint");

promise_test(async t => {
  const log = await runInDataURLDocument(t, (childWindow, report) => {
    const log = [];
    const host = childWindow.document.body.appendChild(childWindow.document.createElement("div"));
    const slot = host.attachShadow({ mode: "open" }).appendChild(childWindow.document.createElement("slot"));
    slot.addEventListener("slotchange", () => log.push("slotchange"));
    host.append("text");
    queueMicrotask(() => {
      log.push("microtask");
      report(log);
    });
  });
  assert_array_equals(log, ["slotchange", "microtask"]);
}, "slotchange in the about:blank child fires in the parent's microtask checkpoint");
