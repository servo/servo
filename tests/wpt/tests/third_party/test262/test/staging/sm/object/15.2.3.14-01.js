/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  Object.keys(O)
info: bugzilla.mozilla.org/show_bug.cgi?id=307791
esid: pending
---*/

assert.sameValue(Object.keys.length, 1);

var o, keys;

o = { a: 3, b: 2 };
keys = Object.keys(o);
assert.compareArray(keys, ["a", "b"]);

o = { get a() { return 17; }, b: 2 };
keys = Object.keys(o),
assert.compareArray(keys, ["a", "b"]);

o = { __iterator__: function() { throw new Error("non-standard __iterator__ called?"); } };
keys = Object.keys(o);
assert.compareArray(keys, ["__iterator__"]);

o = { a: 1, b: 2 };
delete o.a;
o.a = 3;
keys = Object.keys(o);
assert.compareArray(keys, ["b", "a"]);

o = [0, 1, 2];
keys = Object.keys(o);
assert.compareArray(keys, ["0", "1", "2"]);

o = /./.exec("abc");
keys = Object.keys(o);
assert.compareArray(keys, ["0", "index", "input", "groups"]);

o = { a: 1, b: 2, c: 3 };
delete o.b;
o.b = 5;
keys = Object.keys(o);
assert.compareArray(keys, ["a", "c", "b"]);

function f() { }
f.prototype.p = 1;
o = new f();
o.g = 1;
keys = Object.keys(o);
assert.compareArray(keys, ["g"]);
