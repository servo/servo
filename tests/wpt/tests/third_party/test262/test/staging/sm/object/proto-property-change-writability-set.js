/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Don't assert anything about a shape from the property cache until it's known the cache entry matches
info: bugzilla.mozilla.org/show_bug.cgi?id=713944
esid: pending
---*/

var accDesc = { set: function() {} };
var dataDesc = { value: 3 };

function f()
{
  propertyIsEnumerable = {};
}
function g()
{
  propertyIsEnumerable = {};
}

Object.defineProperty(Object.prototype, "propertyIsEnumerable", accDesc);
f();
Object.defineProperty(Object.prototype, "propertyIsEnumerable", dataDesc);
assert.sameValue(propertyIsEnumerable, 3);
f();
assert.sameValue(propertyIsEnumerable, 3);
g();
assert.sameValue(propertyIsEnumerable, 3);



var a = { p1: 1, p2: 2 };
var b = Object.create(a);
Object.defineProperty(a, "p1", {set: function () {}});
for (var i = 0; i < 2; i++)
{
  b.p1 = {};
  Object.defineProperty(a, "p1", {value: 3});
}
assert.sameValue(b.p1, 3);
assert.sameValue(a.p1, 3);
