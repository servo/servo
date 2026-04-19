/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  String.prototype.match must define matches on the returned array, not set them
info: bugzilla.mozilla.org/show_bug.cgi?id=677820
esid: pending
---*/

var called = false;
function setterFunction(v) { called = true; }
function getterFunction(v) { return "getter"; }

Object.defineProperty(Array.prototype, 1,
  { get: getterFunction, set: setterFunction });

assert.sameValue(called, false);
var matches = "abcdef".match(/./g);
assert.sameValue(called, false);
assert.sameValue(matches.length, 6);
assert.sameValue(matches[0], "a");
assert.sameValue(matches[1], "b");
assert.sameValue(matches[2], "c");
assert.sameValue(matches[3], "d");
assert.sameValue(matches[4], "e");
assert.sameValue(matches[5], "f");

var desc = Object.getOwnPropertyDescriptor(Array.prototype, 1);
assert.sameValue(desc.get, getterFunction);
assert.sameValue(desc.set, setterFunction);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.configurable, false);
assert.sameValue([][1], "getter");

assert.sameValue(called, false);
