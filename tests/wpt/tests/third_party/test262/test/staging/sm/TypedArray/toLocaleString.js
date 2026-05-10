// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/
const TypedArrayPrototype = Object.getPrototypeOf(Int8Array.prototype);

// %TypedArrayPrototype% has an own "toLocaleString" function property.
assert.sameValue(TypedArrayPrototype.hasOwnProperty("toLocaleString"), true);
assert.sameValue(typeof TypedArrayPrototype.toLocaleString, "function");

// The initial value of %TypedArrayPrototype%.toLocaleString is not Array.prototype.toLocaleString.
assert.sameValue(TypedArrayPrototype.toLocaleString === Array.prototype.toLocaleString, false);

// The concrete TypedArray prototypes do not have an own "toLocaleString" property.
assert.sameValue(anyTypedArrayConstructors.every(c => !c.hasOwnProperty("toLocaleString")), true);

verifyProperty(TypedArrayPrototype, "toLocaleString", {
    value: TypedArrayPrototype.toLocaleString,
    writable: true,
    enumerable: false,
    configurable: true,
}, {
    restore: true
});

assert.sameValue(TypedArrayPrototype.toLocaleString.name, "toLocaleString");
assert.sameValue(TypedArrayPrototype.toLocaleString.length, 0);

// It's not a generic method.
assert.throws(TypeError, () => TypedArrayPrototype.toLocaleString.call());
for (let invalid of [void 0, null, {}, [], function(){}, true, 0, "", Symbol()]) {
    assert.throws(TypeError, () => TypedArrayPrototype.toLocaleString.call(invalid));
}

const localeOne = 1..toLocaleString(),
      localeTwo = 2..toLocaleString(),
      localeSep = [,,].toLocaleString();

for (let constructor of anyTypedArrayConstructors) {
    assert.sameValue(new constructor([]).toLocaleString(), "");
    assert.sameValue(new constructor([1]).toLocaleString(), localeOne);
    assert.sameValue(new constructor([1, 2]).toLocaleString(), localeOne + localeSep + localeTwo);
}

const originalNumberToLocaleString = Number.prototype.toLocaleString;

// Calls Number.prototype.toLocaleString on each element.
for (let constructor of anyTypedArrayConstructors) {
    Number.prototype.toLocaleString = function() {
        "use strict";

        // Ensure this-value is not boxed.
        assert.sameValue(typeof this, "number");

        // Test ToString is applied.
        return {
            valueOf: () => {
                throw new Error("valueOf called");
            },
            toString: () => {
                return this + 10;
            }
        };
    };
    let typedArray = new constructor([1, 2]);
    assert.sameValue(typedArray.toLocaleString(), "11" + localeSep + "12");
}
Number.prototype.toLocaleString = originalNumberToLocaleString;

// Calls Number.prototype.toLocaleString from the current Realm.
const otherGlobal = $262.createRealm().global;
for (let constructor of anyTypedArrayConstructors) {
    Number.prototype.toLocaleString = function() {
        "use strict";
        called = true;
        return this;
    };
    let typedArray = new otherGlobal[constructor.name]([1]);
    let called = false;
    assert.sameValue(TypedArrayPrototype.toLocaleString.call(typedArray), "1");
    assert.sameValue(called, true);
}
Number.prototype.toLocaleString = originalNumberToLocaleString;
