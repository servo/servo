/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

function C(){}
C.prototype = 1;

assert.throws(TypeError, function() {
  Object.defineProperty(C, "prototype", {get: function() { throw 0; }});
});

new C; // don't assert
