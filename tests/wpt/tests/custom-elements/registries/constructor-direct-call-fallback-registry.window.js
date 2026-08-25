// https://html.spec.whatwg.org/multipage/custom-elements.html#html-element-constructors
// https://dom.spec.whatwg.org/#concept-create-element

function createShadowForTest(t, customElementRegistry) {
  const host = document.createElement('div');
  const shadow = host.attachShadow({mode: 'open', customElementRegistry});
  document.body.appendChild(host);
  t.add_cleanup(() => host.remove());
  return shadow;
}

// GlobalOnly is only defined in the global registry, so `new GlobalOnly()` must
// resolve against it even during a scoped upgrade.
class GlobalOnly extends HTMLElement {}
window.customElements.define('global-only-element', GlobalOnly);

test(t => {
  let nestedElement;
  let nestedError;
  class ScopedElement extends HTMLElement {
    constructor() {
      super();
      try {
        nestedElement = new GlobalOnly();
      } catch (e) {
        nestedError = e;
      }
    }
  }

  const registry = new CustomElementRegistry();
  registry.define('scoped-element-1', ScopedElement);

  const shadow = createShadowForTest(t, registry);
  shadow.innerHTML = '<scoped-element-1></scoped-element-1>';

  const shadowElement = shadow.firstChild;
  assert_true(shadowElement instanceof ScopedElement);
  assert_equals(shadowElement.localName, 'scoped-element-1');

  assert_equals(nestedError, undefined,
      'new GlobalOnly() should not throw during a scoped upgrade');
  assert_true(nestedElement instanceof GlobalOnly);
  assert_equals(nestedElement.localName, 'global-only-element');
}, 'Direct construction of a global-only element after super() during a scoped upgrade');

test(t => {
  let nestedElement;
  let nestedError;
  class ScopedElement extends HTMLElement {
    constructor() {
      try {
        nestedElement = new GlobalOnly();
      } catch (e) {
        nestedError = e;
      }
      super();
    }
  }

  const registry = new CustomElementRegistry();
  registry.define('scoped-element-2', ScopedElement);

  const shadow = createShadowForTest(t, registry);
  shadow.innerHTML = '<scoped-element-2></scoped-element-2>';

  const shadowElement = shadow.firstChild;
  assert_true(shadowElement instanceof ScopedElement);
  assert_equals(shadowElement.localName, 'scoped-element-2');

  assert_equals(nestedError, undefined,
      'new GlobalOnly() should not throw during a scoped upgrade');
  assert_true(nestedElement instanceof GlobalOnly);
  assert_equals(nestedElement.localName, 'global-only-element');
}, 'Direct construction of a global-only element before super() during a scoped upgrade');
