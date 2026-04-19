// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/

var otherGlobal = $262.createRealm().global;

for (var constructor of anyTypedArrayConstructors) {
    assert.sameValue(constructor.of.length, 0);

    assert.deepEqual(Object.getOwnPropertyDescriptor(constructor.__proto__, "of"), {
        value: constructor.of,
        writable: true,
        enumerable: false,
        configurable: true
    });

    // Basic tests.
    assert.sameValue(constructor.of().constructor, constructor);
    assert.sameValue(constructor.of() instanceof constructor, true);
    assert.deepEqual(constructor.of(10), new constructor([10]));
    assert.deepEqual(constructor.of(1, 2, 3), new constructor([1, 2, 3]));
    assert.deepEqual(constructor.of("1", "2", "3"), new constructor([1, 2, 3]));

    // This method can't be transplanted to other constructors.
    assert.throws(TypeError, () => constructor.of.call(Array));
    assert.throws(TypeError, () => constructor.of.call(Array, 1, 2, 3));

    var hits = 0;
    assert.deepEqual(constructor.of.call(function(len) {
        assert.sameValue(arguments.length, 1);
        assert.sameValue(len, 3);
        hits++;
        return new constructor(len);
    }, 10, 20, 30), new constructor([10, 20, 30]));
    assert.sameValue(hits, 1);

    // Behavior across compartments.
    var newC = otherGlobal[constructor.name];
    assert.sameValue(newC.of() instanceof newC, true);
    assert.sameValue(newC.of() instanceof constructor, false);
    assert.sameValue(newC.of.call(constructor) instanceof constructor, true);

    // Throws if `this` isn't a constructor.
    var invalidConstructors = [undefined, null, 1, false, "", Symbol(), [], {}, /./,
                               constructor.of, () => {}];
    invalidConstructors.forEach(C => {
        assert.throws(TypeError, () => {
            constructor.of.call(C);
        });
    });

    // Throw if `this` is a method definition or a getter/setter function.
    assert.throws(TypeError, () => {
        constructor.of.call({method() {}}.method);
    });
    assert.throws(TypeError, () => {
        constructor.of.call(Object.getOwnPropertyDescriptor({get getter() {}}, "getter").get);
    });

    // Generators are not legal constructors.
    assert.throws(TypeError, () => {
      constructor.of.call(function*(len) {
        return len;
      }, "a")
    });

    // An exception might be thrown in a strict assignment to the new object's indexed properties.
    assert.throws(TypeError, () => {
        constructor.of.call(function() {
            return {get 0() {}};
        }, "a");
    });

    assert.throws(TypeError, () => {
        constructor.of.call(function() {
            return Object("1");
        }, "a");
    });

    assert.throws(TypeError, () => {
        constructor.of.call(function() {
            return Object.create({
                set 0(v) {
                    throw new TypeError;
                }
            });
        }, "a");
    });
}

for (let constructor of anyTypedArrayConstructors.filter(isFloatConstructor)) {
    assert.deepEqual(constructor.of(0.1, null, undefined, NaN), new constructor([0.1, 0, NaN, NaN]));
}

