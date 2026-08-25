/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Deleting .length from a variety of builtin functions works as expected.
for (var fun of [Math.sin, Array.prototype.map, eval]) {
    assert.sameValue(delete fun.length, true);
    assert.sameValue(fun.hasOwnProperty("length"), false);
    assert.sameValue("length" in fun, true);  // still inheriting Function.prototype.length
    assert.sameValue(fun.length, 0);

    // The inherited property is nonwritable, so assigning still fails
    // (silently, in sloppy mode).
    fun.length = Math.hypot;
    assert.sameValue(fun.length, 0);

    // It can be shadowed via defineProperty.
    Object.defineProperty(fun, "length", {value: Math.hypot});
    assert.sameValue(fun.length, Math.hypot);
}

