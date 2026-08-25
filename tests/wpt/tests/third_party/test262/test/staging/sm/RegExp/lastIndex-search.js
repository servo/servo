// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// RegExp.prototype[Symbol.search]: Test lastIndex changes for ES2017.

// RegExp-like class to test the RegExp method slow paths.
class DuckRegExp extends RegExp {
    constructor(pattern, flags) {
        return Object.create(DuckRegExp.prototype, {
            regExp: {
                value: new RegExp(pattern, flags)
            },
            lastIndex: {
                value: 0, writable: true, enumerable: false, configurable: false
            }
        });
    }

    exec(...args) {
        this.regExp.lastIndex = this.lastIndex;
        try {
            return this.regExp.exec(...args);
        } finally {
            if (this.global || this.sticky)
                this.lastIndex = this.regExp.lastIndex;
        }
    }

    get source() { return this.regExp.source; }

    get global() { return this.regExp.global; }
    get ignoreCase() { return this.regExp.ignoreCase; }
    get multiline() { return this.regExp.multiline; }
    get sticky() { return this.regExp.sticky; }
    get unicode() { return this.regExp.unicode; }
}

// Test various combinations of:
// - Pattern matches or doesn't match
// - Global and/or sticky flag is set.
// - lastIndex exceeds the input string length
// - lastIndex is +-0
const testCasesNotPositiveZero = [
    { regExp: /a/,  lastIndex: -1, input: "a" },
    { regExp: /a/g, lastIndex: -1, input: "a" },
    { regExp: /a/y, lastIndex: -1, input: "a" },

    { regExp: /a/,  lastIndex: 100, input: "a" },
    { regExp: /a/g, lastIndex: 100, input: "a" },
    { regExp: /a/y, lastIndex: 100, input: "a" },

    { regExp: /a/,  lastIndex: -1, input: "b" },
    { regExp: /a/g, lastIndex: -1, input: "b" },
    { regExp: /a/y, lastIndex: -1, input: "b" },

    { regExp: /a/,  lastIndex: -0, input: "a" },
    { regExp: /a/g, lastIndex: -0, input: "a" },
    { regExp: /a/y, lastIndex: -0, input: "a" },

    { regExp: /a/,  lastIndex: -0, input: "b" },
    { regExp: /a/g, lastIndex: -0, input: "b" },
    { regExp: /a/y, lastIndex: -0, input: "b" },
];

const testCasesPositiveZero = [
    { regExp: /a/,  lastIndex: 0, input: "a" },
    { regExp: /a/g, lastIndex: 0, input: "a" },
    { regExp: /a/y, lastIndex: 0, input: "a" },

    { regExp: /a/,  lastIndex: 0, input: "b" },
    { regExp: /a/g, lastIndex: 0, input: "b" },
    { regExp: /a/y, lastIndex: 0, input: "b" },
];

const testCases = [...testCasesNotPositiveZero, ...testCasesPositiveZero];

for (let Constructor of [RegExp, DuckRegExp]) {
    // Basic test.
    for (let {regExp, lastIndex, input} of testCases) {
        let re = new Constructor(regExp);
        re.lastIndex = lastIndex;
        re[Symbol.search](input);
        assert.sameValue(re.lastIndex, lastIndex);
    }

    // Test when lastIndex is non-writable and not positive zero.
    for (let {regExp, lastIndex, input} of testCasesNotPositiveZero) {
        let re = new Constructor(regExp);
        Object.defineProperty(re, "lastIndex", { value: lastIndex, writable: false });
        assert.throws(TypeError, () => re[Symbol.search](input));
        assert.sameValue(re.lastIndex, lastIndex);
    }

    // Test when lastIndex is non-writable and positive zero.
    for (let {regExp, lastIndex, input} of testCasesPositiveZero) {
        let re = new Constructor(regExp);
        Object.defineProperty(re, "lastIndex", { value: lastIndex, writable: false });
        if (re.global || re.sticky) {
            assert.throws(TypeError, () => re[Symbol.search](input));
        } else {
            re[Symbol.search](input);
        }
        assert.sameValue(re.lastIndex, lastIndex);
    }

    // Test lastIndex isn't converted to a number.
    for (let {regExp, lastIndex, input} of testCases) {
        let re = new RegExp(regExp);
        let badIndex = {
            valueOf() {
                assert.sameValue(false, true);
            }
        };
        re.lastIndex = badIndex;
        re[Symbol.search](input);
        assert.sameValue(re.lastIndex, badIndex);
    }
}

