/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Implement Object.preventExtensions, Object.isExtensible
info: bugzilla.mozilla.org/show_bug.cgi?id=492849
esid: pending
---*/

assert.sameValue(typeof Object.isExtensible, "function");
assert.sameValue(Object.isExtensible.length, 1);

var slowArray = [1, 2, 3];
slowArray.slow = 5;
var objs =
  [{}, { 1: 2 }, { a: 3 }, [], [1], [, 1], slowArray, function a(){}, /a/];

for (var i = 0, sz = objs.length; i < sz; i++)
{
  var o = objs[i];
  assert.sameValue(Object.isExtensible(o), true, "object " + i + " not extensible?");

  var o2 = Object.preventExtensions(o);
  assert.sameValue(o, o2);

  assert.sameValue(Object.isExtensible(o), false, "object " + i + " is extensible?");
}
