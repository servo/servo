/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  ArrayBuffer cannot access properties defined on the prototype chain.
info: bugzilla.mozilla.org/show_bug.cgi?id=665961
esid: pending
---*/

ArrayBuffer.prototype.prop = "on prototype";
var b = new ArrayBuffer([]);
assert.sameValue(b.prop, "on prototype");

var c = new ArrayBuffer([]);
assert.sameValue(c.prop, "on prototype");
c.prop = "direct";
assert.sameValue(c.prop, "direct");

assert.sameValue(ArrayBuffer.prototype.prop, "on prototype");
assert.sameValue(new ArrayBuffer([]).prop, "on prototype");

assert.sameValue(c.nonexistent, undefined);
