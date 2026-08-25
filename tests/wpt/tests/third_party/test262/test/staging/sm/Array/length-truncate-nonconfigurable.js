/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array length redefinition behavior with non-configurable elements
info: bugzilla.mozilla.org/show_bug.cgi?id=858381
esid: pending
---*/

var arr = [0, 1, 2];
Object.defineProperty(arr, 1, { configurable: false });

assert.throws(TypeError, function() {
  Object.defineProperty(arr, "length", { value: 0, writable: false });
}, "must throw TypeError when array truncation would have to remove non-configurable elements");

assert.sameValue(arr.length, 2, "length is highest remaining index plus one");

var desc = Object.getOwnPropertyDescriptor(arr, "length");
assert.sameValue(desc !== undefined, true);

assert.sameValue(desc.value, 2);
assert.sameValue(desc.writable, false);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.configurable, false);
