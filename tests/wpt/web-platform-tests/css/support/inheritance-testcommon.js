'use strict';

(function() {

function assert_initial(property, initial) {
  test(() => {
    const target = document.getElementById('target');
    if (!getComputedStyle(target)[property])
      return;
    target.style[property] = 'initial';
    assert_equals(getComputedStyle(target)[property], initial);
    target.style[property] = '';
  }, 'Property ' + property + ' has initial value ' + initial);
}

/**
 * Create tests that a CSS property inherits and has the given initial value.
 *
 * The current document must have an element #target within element #container.
 *
 * @param {string} property  The name of the CSS property being tested.
 * @param {string} initial   The computed value for 'initial'.
 * @param {string} other     An arbitrary value for the property that round
 *                           trips and is distinct from the initial value.
 */
function assert_inherited(property, initial, other) {
  assert_initial(property, initial);

  test(() => {
    const container = document.getElementById('container');
    const target = document.getElementById('target');
    if (!getComputedStyle(target)[property])
      return;
    container.style[property] = 'initial';
    target.style[property] = 'unset';
    assert_not_equals(getComputedStyle(container)[property], other);
    assert_not_equals(getComputedStyle(target)[property], other);
    container.style[property] = other;
    assert_equals(getComputedStyle(container)[property], other);
    assert_equals(getComputedStyle(target)[property], other);
    target.style[property] = 'initial';
    assert_equals(getComputedStyle(container)[property], other);
    assert_not_equals(getComputedStyle(target)[property], other);
    target.style[property] = 'inherit';
    assert_equals(getComputedStyle(target)[property], other);
    container.style[property] = '';
    target.style[property] = '';
  }, 'Property ' + property + ' inherits');
}

/**
 * Create tests that a CSS property does not inherit, and that it has the
 * given initial value.
 *
 * The current document must have an element #target within element #container.
 *
 * @param {string} property  The name of the CSS property being tested.
 * @param {string} initial   The computed value for 'initial'.
 * @param {string} other     An arbitrary value for the property that round
 *                           trips and is distinct from the initial value.
 */
function assert_not_inherited(property, initial, other) {
  assert_initial(property, initial);

  test(() => {
    const container = document.getElementById('container');
    const target = document.getElementById('target');
    if (!getComputedStyle(target)[property])
      return;
    container.style[property] = 'initial';
    target.style[property] = 'unset';
    assert_not_equals(getComputedStyle(container)[property], other);
    assert_not_equals(getComputedStyle(target)[property], other);
    container.style[property] = other;
    assert_equals(getComputedStyle(container)[property], other);
    assert_not_equals(getComputedStyle(target)[property], other);
    target.style[property] = 'inherit';
    assert_equals(getComputedStyle(target)[property], other);
    container.style[property] = '';
    target.style[property] = '';
  }, 'Property ' + property + ' does not inherit');
}

window.assert_inherited = assert_inherited;
window.assert_not_inherited = assert_not_inherited;
})();
