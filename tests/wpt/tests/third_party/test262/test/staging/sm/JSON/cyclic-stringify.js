/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Properly detect cycles in JSON.stringify (throw TypeError, check for cycles rather than imprecisely rely on recursion limits)
info: bugzilla.mozilla.org/show_bug.cgi?id=578273
esid: pending
---*/

// objects

var count = 0;
var desc =
  {
    get: function() { count++; return obj; },
    enumerable: true,
    configurable: true
  };
var obj = Object.defineProperty({ p1: 0 }, "p2", desc);

assert.throws(TypeError, function() {
  JSON.stringify(obj);
});
assert.sameValue(count, 1, "cyclic data structures not detected immediately");

count = 0;
var obj2 = Object.defineProperty({}, "obj", desc);
assert.throws(TypeError, function() {
  JSON.stringify(obj2);
});
assert.sameValue(count, 2, "cyclic data structures not detected immediately");


// arrays

var count = 0;
var desc =
  {
    get: function() { count++; return arr; },
    enumerable: true,
    configurable: true
  };
var arr = Object.defineProperty([], "0", desc);

assert.throws(TypeError, function() {
  JSON.stringify(arr);
});
assert.sameValue(count, 1, "cyclic data structures not detected immediately");

count = 0;
var arr2 = Object.defineProperty([], "0", desc);
assert.throws(TypeError, function() {
  JSON.stringify(arr2);
});
assert.sameValue(count, 2, "cyclic data structures not detected immediately");
