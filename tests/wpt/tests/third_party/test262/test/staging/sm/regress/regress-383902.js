/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var c = 0;

function f(a) {
  const b = a;
  try {
    eval('"use strict"; b = 1 + a; c = 1');
    assert.sameValue(0, 1);
  } catch (e) {
    assert.sameValue(e.name, 'TypeError');
    assert.sameValue(0, c);
    assert.sameValue(a, b);
  }
}

var w = 42;
f(w);

c = 2;
try {
  eval('"use strict"; function g(x) { const y = x; y = 1 + x; } c = 3');
} catch (e) {
  assert.sameValue(0, 1);
}

c = 4;
try {
  eval('"use strict"; const z = w; z = 1 + w; c = 5');
  assert.sameValue(0, 1);
} catch (e) {
  assert.sameValue(e.name, 'TypeError');
  assert.sameValue(4, c);
  assert.sameValue('z' in this, false);
}

