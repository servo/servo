/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  String.prototype.match should zero the .lastIndex when called with a global RegExp
info: bugzilla.mozilla.org/show_bug.cgi?id=501739
esid: pending
---*/

var s = '0x2x4x6x8';
var p = /x/g;
p.lastIndex = 3;

var arr = s.match(p);
assert.sameValue(arr.length, 4);
arr.forEach(function(v) { assert.sameValue(v, "x"); });
assert.sameValue(p.lastIndex, 0);
