// https://dom.spec.whatwg.org/#concept-attach-a-shadow-root

function defineShadowDisabled(registry, name) {
  registry.define(name, class extends HTMLElement {
    static disabledFeatures = ["shadow"];
  });
}

function attachShadowRootWithRegistry(registry) {
  return document.createElement("div").attachShadow({ mode: "open", customElementRegistry: registry });
}

test(() => {
  const registry = new CustomElementRegistry();
  defineShadowDisabled(registry, "scoped-disabled-element");
  const element = document.createElement("scoped-disabled-element", { customElementRegistry: registry });
  assert_equals(element.customElementRegistry, registry);
  assert_throws_dom("NotSupportedError", () => element.attachShadow({ mode: "open" }));
}, "attachShadow() throws when the element's scoped registry disables shadow for its name");

test(t => {
  const registry = new CustomElementRegistry();
  defineShadowDisabled(registry, "connected-scoped-disabled-element");
  const element = document.createElement("connected-scoped-disabled-element", { customElementRegistry: registry });
  document.body.append(element);
  t.add_cleanup(() => element.remove());
  assert_equals(element.customElementRegistry, registry);
  assert_throws_dom("NotSupportedError", () => element.attachShadow({ mode: "open" }));
}, "attachShadow() throws when the element's scoped registry disables shadow for its name, after the element is appended to the document");

test(() => {
  defineShadowDisabled(customElements, "global-disabled-element");
  const element = document.createElement("global-disabled-element", { customElementRegistry: new CustomElementRegistry() });
  element.attachShadow({ mode: "open" });
  assert_not_equals(element.shadowRoot, null);
}, "attachShadow() ignores the global registry for an element with a scoped registry");

test(() => {
  const registry = new CustomElementRegistry();
  defineShadowDisabled(registry, "shadow-tree-scoped-disabled-element");
  const shadowRoot = attachShadowRootWithRegistry(registry);
  shadowRoot.innerHTML = "<shadow-tree-scoped-disabled-element></shadow-tree-scoped-disabled-element>";
  const element = shadowRoot.firstChild;
  assert_equals(element.customElementRegistry, registry);
  assert_throws_dom("NotSupportedError", () => element.attachShadow({ mode: "open" }));
}, "attachShadow() throws when the scoped registry of the element's shadow tree disables shadow for its name");

test(() => {
  defineShadowDisabled(customElements, "shadow-tree-global-disabled-element");
  const shadowRoot = attachShadowRootWithRegistry(new CustomElementRegistry());
  shadowRoot.innerHTML = "<shadow-tree-global-disabled-element></shadow-tree-global-disabled-element>";
  const element = shadowRoot.firstChild;
  element.attachShadow({ mode: "open" });
  assert_not_equals(element.shadowRoot, null);
}, "attachShadow() ignores the global registry for an element in a shadow tree with a scoped registry");

test(() => {
  defineShadowDisabled(customElements, "appended-global-disabled-element");
  const element = document.createElement("appended-global-disabled-element");
  attachShadowRootWithRegistry(new CustomElementRegistry()).append(element);
  assert_equals(element.customElementRegistry, customElements);
  assert_throws_dom("NotSupportedError", () => element.attachShadow({ mode: "open" }));
}, "attachShadow() throws when the global registry disables shadow for the name of an element that keeps it in a shadow tree with a scoped registry");

test(() => {
  defineShadowDisabled(customElements, "null-registry-element");
  const shadowRoot = attachShadowRootWithRegistry(null);
  shadowRoot.innerHTML = "<null-registry-element></null-registry-element>";
  const element = shadowRoot.firstChild;
  assert_equals(element.customElementRegistry, null);
  element.attachShadow({ mode: "open" });
  assert_not_equals(element.shadowRoot, null);
}, "attachShadow() ignores the global registry for an element with a null registry");
