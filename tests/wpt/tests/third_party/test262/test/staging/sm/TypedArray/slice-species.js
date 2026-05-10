// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
for (var constructor of typedArrayConstructors) {
    // Basic tests for our SpeciesConstructor implementation.
    var undefConstructor = new constructor(2);
    undefConstructor.constructor = undefined;
    assert.deepEqual(undefConstructor.slice(1), new constructor(1));

    assert.throws(TypeError, () => {
        var strConstructor = new constructor;
        strConstructor.constructor = "not a constructor";
        strConstructor.slice(123);
    }, "Assert that we have an invalid constructor");

    // If obj.constructor[@@species] is undefined or null then the default
    // constructor is used.
    var mathConstructor = new constructor(8);
    mathConstructor.constructor = Math.sin;
    assert.deepEqual(mathConstructor.slice(4), new constructor(4));

    var undefSpecies = new constructor(2);
    undefSpecies.constructor = { [Symbol.species]: undefined };
    assert.deepEqual(undefSpecies.slice(1), new constructor(1));

    var nullSpecies = new constructor(2);
    nullSpecies.constructor = { [Symbol.species]: null };
    assert.deepEqual(nullSpecies.slice(1), new constructor(1));

    // If obj.constructor[@@species] is different constructor, it should be
    // used.
    for (var constructor2 of typedArrayConstructors) {
        var modifiedConstructor = new constructor(2);
        modifiedConstructor.constructor = constructor2;
        assert.deepEqual(modifiedConstructor.slice(1), new constructor2(1));

        var modifiedSpecies = new constructor(2);
        modifiedSpecies.constructor = { [Symbol.species]: constructor2 };
        assert.deepEqual(modifiedSpecies.slice(1), new constructor2(1));
    }

    // If obj.constructor[@@species] is neither undefined nor null, and it's
    // not a constructor, TypeError should be thrown.
    assert.throws(TypeError, () => {
        var strSpecies = new constructor;
        strSpecies.constructor = { [Symbol.species]: "not a constructor" };
        strSpecies.slice(123);
    });
}

