/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  ({}).toString.call(null) == "[object Null]", ({}).toString.call(undefined) == "[object Undefined]"
info: bugzilla.mozilla.org/show_bug.cgi?id=575522
esid: pending
---*/

var toString = Object.prototype.toString;

assert.sameValue(toString.call(null), "[object Null]");
assert.sameValue(toString.call(undefined), "[object Undefined]");

assert.sameValue(toString.call(true), "[object Boolean]");
assert.sameValue(toString.call(false), "[object Boolean]");

assert.sameValue(toString.call(0), "[object Number]");
assert.sameValue(toString.call(-0), "[object Number]");
assert.sameValue(toString.call(1), "[object Number]");
assert.sameValue(toString.call(-1), "[object Number]");
assert.sameValue(toString.call(NaN), "[object Number]");
assert.sameValue(toString.call(Infinity), "[object Number]");
assert.sameValue(toString.call(-Infinity), "[object Number]");

assert.sameValue(toString.call("foopy"), "[object String]");

assert.sameValue(toString.call({}), "[object Object]");
