/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.freeze([]).pop() must throw a TypeError
info: bugzilla.mozilla.org/show_bug.cgi?id=858381
esid: pending
---*/

assert.throws(TypeError, function() {
  Object.freeze([]).pop();
});
