/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var names = [
    "isConcatSpreadable",
    "iterator",
    "match",
    "replace",
    "search",
    "species",
    "hasInstance",
    "split",
    "toPrimitive",
    "unscopables",
    "asyncIterator"
];

var g = $262.createRealm().global;

for (var name of names) {
    // Well-known symbols exist.
    assert.sameValue(typeof Symbol[name], "symbol");

    // They are never in the registry.
    assert.sameValue(Symbol[name] !== Symbol.for("Symbol." + name), true);

    // They are shared across realms.
    assert.sameValue(Symbol[name], g.Symbol[name]);

    // Descriptor is all false.
    var desc = Object.getOwnPropertyDescriptor(Symbol, name);
    assert.sameValue(typeof desc.value, "symbol");
    assert.sameValue(desc.writable, false);
    assert.sameValue(desc.enumerable, false);
    assert.sameValue(desc.configurable, false);
}

