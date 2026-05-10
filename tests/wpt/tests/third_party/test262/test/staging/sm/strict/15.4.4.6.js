/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-strict-shell.js]
description: |
  pending
esid: pending
---*/
function arr() {
  return Object.defineProperty([1, 2, 3], 2, {configurable: false});
}

function obj() {
  var o = {0: 1, 1: 2, 2: 3, length: 3};
  Object.defineProperty(o, 2, {configurable: false});
  return o;
}

assert.sameValue(testLenientAndStrict('var a = arr(); [a.pop(), a]',
                              raisesException(TypeError),
                              raisesException(TypeError)),
         true);

assert.sameValue(testLenientAndStrict('var o = obj(); [Array.prototype.pop.call(o), o]',
                              raisesException(TypeError),
                              raisesException(TypeError)),
         true);

