/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  RegExp lastIndex property set should not coerce type to number
info: bugzilla.mozilla.org/show_bug.cgi?id=465199
esid: pending
---*/

var called = false;
var o = { valueOf: function() { called = true; return 1; } };
var r = /a/g;
var desc, m;

assert.sameValue(r.lastIndex, 0);

desc = Object.getOwnPropertyDescriptor(r, "lastIndex");
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.value, 0);
assert.sameValue(desc.writable, true);

r.lastIndex = o;

assert.sameValue(r.lastIndex, o);

desc = Object.getOwnPropertyDescriptor(r, "lastIndex");
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.value, o);
assert.sameValue(desc.writable, true);

assert.sameValue(called, false);
assert.sameValue(r.exec("aaaa").length, 1);
assert.sameValue(called, true);
assert.sameValue(r.lastIndex, 2);

desc = Object.getOwnPropertyDescriptor(r, "lastIndex");
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.value, 2);
assert.sameValue(desc.writable, true);


r.lastIndex = -0.2;
assert.sameValue(r.lastIndex, -0.2);

m = r.exec("aaaa");
assert.sameValue(Array.isArray(m), true);
assert.sameValue(m.length, 1);
assert.sameValue(m[0], "a");
assert.sameValue(r.lastIndex, 1);

m = r.exec("aaaa");
assert.sameValue(Array.isArray(m), true);
assert.sameValue(m.length, 1);
assert.sameValue(m[0], "a");
assert.sameValue(r.lastIndex, 2);
