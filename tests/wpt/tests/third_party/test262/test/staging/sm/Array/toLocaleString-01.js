/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.prototype.toLocaleString
info: bugzilla.mozilla.org/show_bug.cgi?id=562446
esid: pending
---*/

var o;

o = { length: 2, 0: 7, 1: { toLocaleString: function() { return "baz" } } };
assert.sameValue(Array.prototype.toLocaleString.call(o), "7,baz");

o = {};
assert.sameValue(Array.prototype.toLocaleString.call(o), "");

var log = '';
var arr = {length: {valueOf: function () { log += "L"; return 2; }},
      0: "x", 1: "z"};
assert.sameValue(Array.prototype.toLocaleString.call(arr), "x,z");
assert.sameValue(log, "L");
