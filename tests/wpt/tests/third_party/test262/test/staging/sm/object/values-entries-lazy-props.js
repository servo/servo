// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
// Return new objects for each test case.
function makeTestCases() {
    // Call the resolve hook for arguments/string objects.
    const resolveIndex = object => 0 in object;

    // Calls the resolve hook for functions.
    const resolveFunction = object => "length" in object;

    const expected = array => ({
        keys: Object.keys(array),
        values: Object.values(array),
        entries: Object.entries(array),
    });

    return [
        // Mapped arguments objects.
        {
            object: function(){ return arguments; }(),
            resolve: resolveIndex,
            ...expected([]),
        },
        {
            object: function(x){ return arguments; }(1),
            resolve: resolveIndex,
            ...expected([1]),
        },
        {
            object: function(x, y, z){ return arguments; }(1, 2, 3),
            resolve: resolveIndex,
            ...expected([1, 2, 3]),
        },

        // Unmapped arguments objects.
        {
            object: function(){ "use strict"; return arguments; }(),
            resolve: resolveIndex,
            ...expected([]),
        },
        {
            object: function(x){ "use strict"; return arguments; }(1),
            resolve: resolveIndex,
            ...expected([1]),
        },
        {
            object: function(x, y, z){ "use strict"; return arguments; }(1, 2, 3),
            resolve: resolveIndex,
            ...expected([1, 2, 3]),
        },

        // String objects.
        { object: new String(""), resolve: resolveIndex, ...expected([]), },
        { object: new String("a"), resolve: resolveIndex, ...expected(["a"]), },
        { object: new String("abc"), resolve: resolveIndex, ...expected(["a","b","c"]), },

        // Function objects.
        { object: function(){}, resolve: resolveFunction, ...expected([]), },
    ];
}

var assertWith = {
    keys: assert.compareArray,
    values: assert.compareArray,
    entries(actual, expected) {
        assert.sameValue(actual.length, expected.length);
        for (let i = 0; i < expected.length; ++i)
            assert.compareArray(actual[i], expected[i]);
    }
};

// Test Object.keys/values/entries on objects with enumerate/resolve hooks.

for (let method of ["keys", "values", "entries"]) {
    let assert = assertWith[method];

    // Call each method twice to test the case when
    // - the enumerate hook wasn't yet called,
    // - the enumerate hook was already called.
    for (let {object, [method]: expected} of makeTestCases()) {
        assert(Object[method](object), expected);
        assert(Object[method](object), expected);
    }

    // Test the case when enumerate wasn't yet called, but a property was already resolved.
    for (let {object, resolve, [method]: expected} of makeTestCases()) {
        resolve(object); // Call the resolve hook.

        assert(Object[method](object), expected);
        assert(Object[method](object), expected);
    }
}

