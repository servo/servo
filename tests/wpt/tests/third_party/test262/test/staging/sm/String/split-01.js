/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  String.prototype.split tests
info: bugzilla.mozilla.org/show_bug.cgi?id=614608
esid: pending
---*/

var order = "";
var o1 = { toString: function() { order += "b"; return "-"; }};
var o2 = { valueOf:  function() { order += "a"; return 1; }};
var res = "xyz-xyz".split(o1, o2);

assert.sameValue(order, "ab");
assert.compareArray(res, ["xyz"]);

assert.compareArray("".split(/.?/), []);
assert.compareArray("abc".split(/\b/), ["abc"]);

assert.compareArray("abc".split(/((()))./, 2), ["",""]);
assert.compareArray("abc".split(/((((()))))./, 9), ["","","","","","","","",""]);

// from ES5 15.5.4.14
assert.compareArray("ab".split(/a*?/), ["a", "b"]);
assert.compareArray("ab".split(/a*/), ["", "b"]);
assert.compareArray("A<B>bold</B>and<CODE>coded</CODE>".split(/<(\/)?([^<>]+)>/),
                    ["A", undefined, "B", "bold", "/", "B", "and", undefined,
                     "CODE", "coded", "/", "CODE", ""]);
