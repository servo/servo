// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// RegExp.prototype.exec: Test lastIndex changes for ES2017.

// Test various combinations of:
// - Pattern matches or doesn't match
// - Global and/or sticky flag is set.
// - lastIndex exceeds the input string length
// - lastIndex is +-0
const testCases = [
    { regExp: /a/,  lastIndex: 0, input: "a", result: 0 },
    { regExp: /a/g, lastIndex: 0, input: "a", result: 1 },
    { regExp: /a/y, lastIndex: 0, input: "a", result: 1 },

    { regExp: /a/,  lastIndex: 0, input: "b", result: 0 },
    { regExp: /a/g, lastIndex: 0, input: "b", result: 0 },
    { regExp: /a/y, lastIndex: 0, input: "b", result: 0 },

    { regExp: /a/,  lastIndex: -0, input: "a", result: -0 },
    { regExp: /a/g, lastIndex: -0, input: "a", result: 1 },
    { regExp: /a/y, lastIndex: -0, input: "a", result: 1 },

    { regExp: /a/,  lastIndex: -0, input: "b", result: -0 },
    { regExp: /a/g, lastIndex: -0, input: "b", result: 0 },
    { regExp: /a/y, lastIndex: -0, input: "b", result: 0 },

    { regExp: /a/,  lastIndex: -1, input: "a", result: -1 },
    { regExp: /a/g, lastIndex: -1, input: "a", result: 1 },
    { regExp: /a/y, lastIndex: -1, input: "a", result: 1 },

    { regExp: /a/,  lastIndex: -1, input: "b", result: -1 },
    { regExp: /a/g, lastIndex: -1, input: "b", result: 0 },
    { regExp: /a/y, lastIndex: -1, input: "b", result: 0 },

    { regExp: /a/,  lastIndex: 100, input: "a", result: 100 },
    { regExp: /a/g, lastIndex: 100, input: "a", result: 0 },
    { regExp: /a/y, lastIndex: 100, input: "a", result: 0 },
];

// Basic test.
for (let {regExp, lastIndex, input, result} of testCases) {
    let re = new RegExp(regExp);
    re.lastIndex = lastIndex;
    re.exec(input);
    assert.sameValue(re.lastIndex, result);
}

// Test when lastIndex is non-writable.
for (let {regExp, lastIndex, input} of testCases) {
    let re = new RegExp(regExp);
    Object.defineProperty(re, "lastIndex", { value: lastIndex, writable: false });
    if (re.global || re.sticky) {
        assert.throws(TypeError, () => re.exec(input));
    } else {
        re.exec(input);
    }
    assert.sameValue(re.lastIndex, lastIndex);
}

// Test when lastIndex is changed to non-writable as a side-effect.
for (let {regExp, lastIndex, input} of testCases) {
    let re = new RegExp(regExp);
    let called = false;
    re.lastIndex = {
        valueOf() {
            assert.sameValue(called, false);
            called = true;
            Object.defineProperty(re, "lastIndex", { value: 9000, writable: false });
            return lastIndex;
        }
    };
    if (re.global || re.sticky) {
        assert.throws(TypeError, () => re.exec(input));
    } else {
        re.exec(input);
    }
    assert.sameValue(re.lastIndex, 9000);
    assert.sameValue(called, true);
}

