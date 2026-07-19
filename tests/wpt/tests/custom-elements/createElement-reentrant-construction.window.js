// create an element does not push onto the construction stack; the constructor
// creates the element. So a re-entrant createElement() for the same element
// yields two independent elements rather than hitting an "already constructed"
// marker.

// A failed construction is reported rather than rethrown by create an element.
setup({allow_uncaught_exception: true});

test(() => {
  let nested;
  let needsReentry = true;
  class Reentrant extends HTMLElement {
    constructor() {
      if (needsReentry) {
        needsReentry = false;
        nested = document.createElement('reentrant-before-super');
      }
      super();
    }
  }
  customElements.define('reentrant-before-super', Reentrant);

  const outer = document.createElement('reentrant-before-super');

  assert_true(nested instanceof Reentrant, 'the re-entrant createElement() constructs an element');
  assert_equals(nested.localName, 'reentrant-before-super');
  assert_true(outer instanceof Reentrant, 'the outer createElement() constructs an element');
  assert_equals(outer.localName, 'reentrant-before-super');
  assert_not_equals(outer, nested, 'the two createElement() calls construct distinct elements');
}, 'Re-entrant createElement() of the same element before super()');

test(() => {
  let nested;
  let needsReentry = true;
  class Reentrant extends HTMLElement {
    constructor() {
      super();
      if (needsReentry) {
        needsReentry = false;
        nested = document.createElement('reentrant-after-super');
      }
    }
  }
  customElements.define('reentrant-after-super', Reentrant);

  const outer = document.createElement('reentrant-after-super');

  assert_true(nested instanceof Reentrant, 'the re-entrant createElement() constructs an element');
  assert_equals(nested.localName, 'reentrant-after-super');
  assert_true(outer instanceof Reentrant, 'the outer createElement() constructs an element');
  assert_equals(outer.localName, 'reentrant-after-super');
  assert_not_equals(outer, nested, 'the two createElement() calls construct distinct elements');
}, 'Re-entrant createElement() of the same element after super()');
