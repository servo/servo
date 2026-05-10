// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    compareArray gracefully handles nullish arguments.
includes: [compareArray.js]
---*/

function assertThrows(func, errorMessage) {
    var caught = false;
    try {
        func();
    } catch (error) {
        caught = true;
        assert.sameValue(error.constructor, Test262Error);
        assert.sameValue(error.message, errorMessage);
    }

    assert(caught, `Expected ${func} to throw, but it didn't.`);
}

assertThrows(() => assert.compareArray(), "Actual argument [undefined] shouldn't be primitive. ");
assertThrows(() => assert.compareArray(null, []), "Actual argument [null] shouldn't be primitive. ");
assertThrows(() => assert.compareArray(null, [], "foo"), "Actual argument [null] shouldn't be primitive. foo");

assertThrows(() => assert.compareArray([]), "Expected argument [undefined] shouldn't be primitive. ");
assertThrows(() => assert.compareArray([], undefined, "foo"), "Expected argument [undefined] shouldn't be primitive. foo");
