/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var symbols = [
    Symbol(),
    Symbol("ok"),
    Symbol.for("dummies"),
    Symbol.iterator
];

for (var sym of symbols) {
    assert.sameValue(sym.valueOf(), sym);
    assert.sameValue(Object(sym).valueOf(), sym);
}

// Any other value throws.
var nonsymbols = [undefined, null, NaN, {}, Symbol.prototype];
for (var nonsym of nonsymbols)
    assert.throws(TypeError, () => Symbol.prototype.valueOf.call(nonsym));

