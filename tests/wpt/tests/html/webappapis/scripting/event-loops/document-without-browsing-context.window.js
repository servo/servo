// https://dom.spec.whatwg.org/#queue-a-mutation-observer-compound-microtask
// https://html.spec.whatwg.org/multipage/scripting.html#appropriate-template-contents-owner-document

// A document without a browsing context is in its creator's agent, so mutation
// observers watching it are notified in that agent's microtask checkpoint.
function assertObserverNotifiedBeforeNextMicrotask(node) {
  const log = [];
  new MutationObserver(() => log.push("observer")).observe(node, { childList: true });
  node.append(new Comment());
  return new Promise(resolve => queueMicrotask(() => {
    log.push("microtask");
    resolve(log);
  })).then(log => assert_array_equals(log, ["observer", "microtask"]));
}

promise_test(() => {
  return assertObserverNotifiedBeforeNextMicrotask(document.createElement("template").content);
}, "Mutation observer watching template contents");

promise_test(() => {
  return assertObserverNotifiedBeforeNextMicrotask(document.implementation.createHTMLDocument().body);
}, "Mutation observer watching a createHTMLDocument() document");

promise_test(() => {
  return assertObserverNotifiedBeforeNextMicrotask(document.implementation.createHTMLDocument("title").body);
}, "Mutation observer watching a createHTMLDocument() document with a title");

promise_test(() => {
  return assertObserverNotifiedBeforeNextMicrotask(new DOMParser().parseFromString("<body>", "text/html").body);
}, "Mutation observer watching a DOMParser document");

promise_test(() => {
  return assertObserverNotifiedBeforeNextMicrotask(new Document());
}, "Mutation observer watching a new Document()");

promise_test(() => {
  return assertObserverNotifiedBeforeNextMicrotask(Document.parseHTMLUnsafe("<body>").body);
}, "Mutation observer watching a Document.parseHTMLUnsafe() document");

promise_test(async () => {
  const xhr = new XMLHttpRequest();
  xhr.open("GET", "/common/blank.html");
  xhr.responseType = "document";
  const loaded = new Promise(resolve => xhr.onload = resolve);
  xhr.send();
  await loaded;
  return assertObserverNotifiedBeforeNextMicrotask(xhr.response.body);
}, "Mutation observer watching an XMLHttpRequest response document");
