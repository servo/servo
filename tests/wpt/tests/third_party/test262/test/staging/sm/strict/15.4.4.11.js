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
  return Object.defineProperty([20, 10, 30], 0, {writable: false});
}

assert.sameValue(testLenientAndStrict('var a = arr(); a.sort()',
                              raisesException(TypeError),
                              raisesException(TypeError)),
         true);

function obj() {
  var o = {0: 20, 1: 10, 2: 30, length: 3};
  Object.defineProperty(o, 0, {writable: false});
  return o;
}

assert.sameValue(testLenientAndStrict('var o = obj(); Array.prototype.sort.call(o)',
                              raisesException(TypeError), 
                              raisesException(TypeError)),
         true);

// The behavior of sort is implementation-defined if the object being
// sorted is sparse, so I'm not sure how to test its behavior on
// non-configurable properties.

