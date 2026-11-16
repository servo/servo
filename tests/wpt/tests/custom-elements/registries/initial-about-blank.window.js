// META: script=/common/gc.js

async_test(t => {
  const blankDocumentURL = new URL("/common/blank.html", location).href;
  const frame = document.createElement("iframe");
  t.add_cleanup(() => frame.remove());
  frame.src = blankDocumentURL;
  document.body.append(frame);
  const initialRegistry = frame.contentWindow.customElements;
  frame.contentWindow.eval(`
    class AA extends HTMLElement { };
    self.globalAA = AA;
    customElements.define("a-a", AA);
    document.body.innerHTML = "<a-a>";
  `);
  assert_equals(frame.contentDocument.body.firstChild.localName, "a-a");
  assert_true(frame.contentDocument.body.firstChild instanceof frame.contentWindow.globalAA);

  frame.onload = t.step_func_done(t => {
    assert_equals(frame.contentDocument.URL, blankDocumentURL);
    assert_equals(frame.contentDocument.body.innerHTML, "");
    frame.contentDocument.body.innerHTML = "<a-a>";
    assert_equals(frame.contentDocument.body.firstChild.localName, "a-a");
    assert_not_equals(frame.contentWindow.globalAA, undefined, "The Window must be reused");
    assert_not_equals(frame.contentWindow.customElements, initialRegistry,
                      "The replacement document must get a new registry");
    assert_equals(frame.contentWindow.customElements.get("a-a"), undefined);
  });
}, "Each navigable document has its own registry");

function appendIframeNavigatingFromInitialAboutBlank(t) {
  const iframe = document.createElement("iframe");
  t.add_cleanup(() => iframe.remove());
  iframe.src = "/common/blank.html";
  const loaded = new Promise(resolve => iframe.onload = resolve);
  document.body.append(iframe);
  iframe.contentWindow.isReused = true;
  return { iframe, loaded };
}

promise_test(async t => {
  const { iframe, loaded } = appendIframeNavigatingFromInitialAboutBlank(t);
  const oldDocument = iframe.contentDocument;
  const oldRegistry = iframe.contentWindow.customElements;

  await loaded;
  assert_true(iframe.contentWindow.isReused, "window is reused");
  assert_equals(oldDocument.customElementRegistry, oldRegistry, "old document's registry");
  assert_not_equals(iframe.contentWindow.customElements, oldRegistry, "window's registry");
  assert_equals(iframe.contentDocument.customElementRegistry, iframe.contentWindow.customElements, "new document's registry");
}, "The initial about:blank document keeps its registry when its window is reused");

promise_test(async t => {
  const { iframe, loaded } = appendIframeNavigatingFromInitialAboutBlank(t);
  const oldDocument = iframe.contentDocument;
  const oldRegistry = iframe.contentWindow.customElements;
  const oldElement = oldDocument.createElement("test-element");
  oldDocument.body.append(oldElement);

  await loaded;
  assert_true(iframe.contentWindow.isReused, "window is reused");
  const newElement = iframe.contentDocument.createElement("test-element");
  iframe.contentDocument.body.append(newElement);

  class TestElement extends iframe.contentWindow.HTMLElement {}
  oldRegistry.define("test-element", TestElement);
  assert_false(oldElement instanceof TestElement, "element in the old document");
  assert_false(newElement instanceof TestElement, "element in the new document");
}, "define() on the initial about:blank document's registry after its window is reused upgrades nothing");

promise_test(async () => {
  const iframe = document.createElement("iframe");
  document.body.append(iframe);
  const contentWindow = iframe.contentWindow;
  const registry = contentWindow.customElements;
  iframe.remove();

  await garbageCollect();
  assert_equals(contentWindow.customElements, registry, "window's registry");
  assert_equals(contentWindow.document.customElementRegistry, registry, "document's registry");
}, "A removed iframe's window keeps its document's registry after garbage collection");
