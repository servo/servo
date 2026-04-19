/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Implement Object.preventExtensions, Object.isExtensible
info: bugzilla.mozilla.org/show_bug.cgi?id=492849
esid: pending
---*/

function tryStrictSetProperty(o, p, v)
{
  assert.sameValue(Object.prototype.hasOwnProperty.call(o, p), false);
  assert.throws(TypeError, function() {
    "use strict";
    o[p] = v;
  });
}

function trySetProperty(o, p, v)
{
  assert.sameValue(Object.prototype.hasOwnProperty.call(o, p), false);

  o[p] = v;

  assert.notSameValue(o[p], v);
  assert.sameValue(p in o, false);
}

function tryDefineProperty(o, p, v)
{
  assert.sameValue(Object.prototype.hasOwnProperty.call(o, p), false);
  assert.throws(TypeError, function() {
    Object.defineProperty(o, p, { value: v });
  });
}

assert.sameValue(typeof Object.preventExtensions, "function");
assert.sameValue(Object.preventExtensions.length, 1);

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

  tryStrictSetProperty(o, "baz", 17);
  trySetProperty(o, "baz", 17);
  tryDefineProperty(o, "baz", 17);
}
