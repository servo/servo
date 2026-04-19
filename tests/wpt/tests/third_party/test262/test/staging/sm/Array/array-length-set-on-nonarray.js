/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't use a shared-permanent inherited property to implement [].length or (function(){}).length
info: bugzilla.mozilla.org/show_bug.cgi?id=548671
esid: pending
---*/

var a = [];
a.p = 1;
var x = Object.create(a);
assert.sameValue(x.length, 0);
assert.sameValue(x.p, 1);
assert.sameValue(a.length, 0);
