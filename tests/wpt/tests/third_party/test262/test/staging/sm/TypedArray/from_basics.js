// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    // 'from' method is identical for all typed array constructors.
    assert.sameValue(anyTypedArrayConstructors[0].from === constructor.from, true);

    // %TypedArray%.from copies arrays.
    var src = new constructor([1, 2, 3]), copy = constructor.from(src);
    assert.sameValue(copy === src, false);
    assert.sameValue(copy instanceof constructor, true);
    assert.deepEqual(copy, src);

    // Non-element properties are not copied.
    var a = new constructor([0, 1]);
    a.name = "lisa";
    assert.deepEqual(constructor.from(a), new constructor([0, 1]));

    // %TypedArray%.from can copy non-iterable objects, if they're array-like.
    src = {0: 0, 1: 1, length: 2};
    copy = constructor.from(src);
    assert.sameValue(copy instanceof constructor, true);
    assert.deepEqual(copy, new constructor([0, 1]));

    // Properties past the .length are not copied.
    src = {0: "0", 1: "1", 2: "two", 9: "nine", name: "lisa", length: 2};
    assert.deepEqual(constructor.from(src), new constructor([0, 1]));

    // If an object has neither an @@iterator method nor .length,
    // then it's treated as zero-length.
    assert.deepEqual(constructor.from({}), new constructor());

    // Primitives will be coerced to primitive wrapper first.
    assert.deepEqual(constructor.from(1), new constructor());
    assert.deepEqual(constructor.from("123"), new constructor([1, 2, 3]));
    assert.deepEqual(constructor.from(true), new constructor());
    assert.deepEqual(constructor.from(Symbol()), new constructor());

    // Source object property order doesn't matter.
    src = {length: 2, 1: "1", 0: "0"};
    assert.deepEqual(constructor.from(src), new constructor([0, 1]));
}

