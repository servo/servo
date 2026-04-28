/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var cases = [
    {sym: Symbol(), str: "Symbol()"},
    {sym: Symbol("ok"), str: "Symbol(ok)"},
    {sym: Symbol("\0"), str: "Symbol(\0)"},
    {sym: Symbol.iterator, str: "Symbol(Symbol.iterator)"},
    {sym: Symbol.for("dummies"), str: "Symbol(dummies)"}
];

// Symbol.prototype.toString works on both primitive symbols and Symbol
// objects.
for (var test of cases) {
    assert.sameValue(test.sym.toString(), test.str);
    assert.sameValue(Object(test.sym).toString(), test.str);
}

// Any other value throws.
var nonsymbols = [
    undefined, null, "not-ok", new String("still-not-ok"), {}, []
];
for (var nonsym of nonsymbols)
    assert.throws(TypeError, () => Symbol.prototype.toString.call(nonsym));

