// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
function assertSameEntries(actual, expected) {
    assert.sameValue(actual.length, expected.length);
    for (let i = 0; i < expected.length; ++i)
        assert.compareArray(actual[i], expected[i]);
}

// Test Object.keys/values/entries on objects with indexed properties.

// Empty dense elements, test with array and plain object.
{
    let array = [];
    assert.compareArray(Object.keys(array), []);
    assert.compareArray(Object.values(array), []);
    assertSameEntries(Object.entries(array), []);

    let object = {};
    assert.compareArray(Object.keys(object), []);
    assert.compareArray(Object.values(object), []);
    assertSameEntries(Object.entries(object), []);
}

// Dense elements only, test with array and plain object.
{
    let array = [1, 2, 3];
    assert.compareArray(Object.keys(array), ["0", "1", "2"]);
    assert.compareArray(Object.values(array), [1, 2, 3]);
    assertSameEntries(Object.entries(array), [["0", 1], ["1", 2], ["2", 3]]);

    let object = {0: 4, 1: 5, 2: 6};
    assert.compareArray(Object.keys(object), ["0", "1", "2"]);
    assert.compareArray(Object.values(object), [4, 5, 6]);
    assertSameEntries(Object.entries(object), [["0", 4], ["1", 5], ["2", 6]]);
}

// Extra indexed properties only, test with array and plain object.
{
    let array = [];
    Object.defineProperty(array, 0, {configurable: true, enumerable: true, value: 123});

    assert.compareArray(Object.keys(array), ["0"]);
    assert.compareArray(Object.values(array), [123]);
    assertSameEntries(Object.entries(array), [["0", 123]]);

    let object = [];
    Object.defineProperty(object, 0, {configurable: true, enumerable: true, value: 123});

    assert.compareArray(Object.keys(object), ["0"]);
    assert.compareArray(Object.values(object), [123]);
    assertSameEntries(Object.entries(object), [["0", 123]]);
}

// Dense and extra indexed properties, test with array and plain object.
{
    let array = [1, 2, 3];
    Object.defineProperty(array, 0, {writable: false});

    assert.compareArray(Object.keys(array), ["0", "1", "2"]);
    assert.compareArray(Object.values(array), [1, 2, 3]);
    assertSameEntries(Object.entries(array), [["0", 1], ["1", 2], ["2", 3]]);

    let object = {0: 4, 1: 5, 2: 6};
    Object.defineProperty(object, 0, {writable: false});

    assert.compareArray(Object.keys(object), ["0", "1", "2"]);
    assert.compareArray(Object.values(object), [4, 5, 6]);
    assertSameEntries(Object.entries(object), [["0", 4], ["1", 5], ["2", 6]]);
}


