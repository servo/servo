// https://html.spec.whatwg.org/multipage/custom-elements.html#active-custom-element-constructor-map
// https://html.spec.whatwg.org/multipage/dom.html#html-element-constructors

setup({ allow_uncaught_exception: true });

function appendIframe(t) {
  const iframe = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => iframe.remove());
  return iframe;
}

test(t => {
  const iframe = appendIframe(t);
  const element = iframe.contentDocument.body.appendChild(iframe.contentDocument.createElement("global-registry-element"));
  class ThisWindowElement extends HTMLElement {}
  iframe.contentWindow.customElements.define("global-registry-element", ThisWindowElement);
  assert_true(element instanceof ThisWindowElement, "upgraded");
  assert_true(element.matches(":defined"), ":defined");
}, "Upgrade with another window's global registry and a constructor from this window");

test(t => {
  const iframe = appendIframe(t);
  const registry = new CustomElementRegistry();
  const element = document.body.appendChild(document.createElement("scoped-registry-element", { customElementRegistry: registry }));
  t.add_cleanup(() => element.remove());
  class OtherWindowElement extends iframe.contentWindow.HTMLElement {}
  registry.define("scoped-registry-element", OtherWindowElement);
  assert_true(element instanceof OtherWindowElement, "upgraded");
  assert_true(element.matches(":defined"), ":defined");
}, "Upgrade with a scoped registry and a constructor from another window");

test(t => {
  const iframe = appendIframe(t);
  class ThisWindowElement extends HTMLElement {}
  iframe.contentWindow.customElements.define("constructed-element", ThisWindowElement);
  assert_throws_js(TypeError, () => new ThisWindowElement());
}, "Constructing directly uses the registry of the HTMLElement constructor's window");
