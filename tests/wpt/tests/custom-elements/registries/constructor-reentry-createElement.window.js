// Exercises the active custom element constructor map through the DOM
// "create an element" path rather than the HTML "upgrade an element" path.
// https://github.com/whatwg/html/issues/12691

// create an element reports constructor exceptions rather than rethrowing them.
setup({allow_uncaught_exception: true});

test(() => {
  let nestedElement;
  let needsReentry = true;
  class Reentry extends HTMLElement {
    constructor() {
      super();
      if (needsReentry) {
        needsReentry = false;
        nestedElement = document.createElement('global-reentry');
      }
    }
  }
  window.customElements.define('global-reentry', Reentry);

  const registry = new CustomElementRegistry();
  registry.define('scoped-reentry', Reentry);

  const element = document.createElement('scoped-reentry', {customElementRegistry: registry});

  assert_true(element instanceof Reentry);
  assert_equals(element.localName, 'scoped-reentry');
  assert_true(nestedElement instanceof Reentry);
  assert_equals(nestedElement.localName, 'global-reentry');
}, 'Re-entry via createElement with a different registry after super()');

// The re-entrant construction overwrites the map entry for the shared
// constructor and must restore it, or the outer super() falls back to the wrong
// registry.
test(() => {
  let nestedElement;
  let needsReentry = true;
  class Reentry extends HTMLElement {
    constructor() {
      if (needsReentry) {
        needsReentry = false;
        nestedElement = document.createElement('global-reentry-before');
      }
      super();
    }
  }
  window.customElements.define('global-reentry-before', Reentry);

  const registry = new CustomElementRegistry();
  registry.define('scoped-reentry-before', Reentry);

  const element = document.createElement('scoped-reentry-before', {customElementRegistry: registry});

  assert_true(element instanceof Reentry);
  assert_equals(element.localName, 'scoped-reentry-before');
  assert_true(nestedElement instanceof Reentry);
  assert_equals(nestedElement.localName, 'global-reentry-before');
  assert_not_equals(element, nestedElement);
}, 'Re-entry via createElement with a different registry before super()');

// GlobalOnly is a different constructor, so it has its own construction stack
// and resolves against the global registry.
class GlobalOnly extends HTMLElement {}
window.customElements.define('global-only-element', GlobalOnly);

test(() => {
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
  registry.define('scoped-element-after', ScopedElement);

  const element = document.createElement('scoped-element-after', {customElementRegistry: registry});

  assert_true(element instanceof ScopedElement);
  assert_equals(element.localName, 'scoped-element-after');
  assert_equals(nestedError, undefined,
      'new GlobalOnly() should not throw during a scoped createElement construction');
  assert_true(nestedElement instanceof GlobalOnly);
  assert_equals(nestedElement.localName, 'global-only-element');
}, 'Direct construction of a global-only element after super() during a scoped createElement construction');

test(() => {
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
  registry.define('scoped-element-before', ScopedElement);

  const element = document.createElement('scoped-element-before', {customElementRegistry: registry});

  assert_true(element instanceof ScopedElement);
  assert_equals(element.localName, 'scoped-element-before');
  assert_equals(nestedError, undefined,
      'new GlobalOnly() should not throw during a scoped createElement construction');
  assert_true(nestedElement instanceof GlobalOnly);
  assert_equals(nestedElement.localName, 'global-only-element');
}, 'Direct construction of a global-only element before super() during a scoped createElement construction');
