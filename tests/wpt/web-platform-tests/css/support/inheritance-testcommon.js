'use strict';

function assert_initial(property, initial) {
  test(() => {
    if (!getComputedStyle(target)[property])
      return;
    target.style[property] = 'initial';
    assert_equals(getComputedStyle(target)[property], initial);
    target.style[property] = '';
  }, 'Property ' + property + ' has initial value ' + initial);
}

function assert_inherited(property, initial, other) {
  assert_initial(property, initial);

  test(() => {
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

function assert_not_inherited(property, initial, other) {
  assert_initial(property, initial);

  test(() => {
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
