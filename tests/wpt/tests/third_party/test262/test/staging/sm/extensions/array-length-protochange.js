/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't use a shared-permanent inherited property to implement [].length or (function(){}).length
info: bugzilla.mozilla.org/show_bug.cgi?id=548671
esid: pending
---*/

var a = [1, 2, 3];
a.__proto__ = null;
assert.sameValue("length" in a, true, "length should be own property of array");
assert.sameValue(Object.hasOwnProperty.call(a, "length"), true,
              "length should be own property of array");
assert.sameValue(a.length, 3, "array length should be 3");

var a = [], b = [];
b.__proto__ = a;
assert.sameValue(b.hasOwnProperty("length"), true,
              "length should be own property of array");
b.length = 42;
assert.sameValue(b.length, 42, "should have mutated b's (own) length");
assert.sameValue(a.length, 0, "should not have mutated a's (own) length");
