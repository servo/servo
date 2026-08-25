// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// RegExp.prototype[Symbol.match, Symbol.replace]: Test lastIndex changes for ES2017.

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

    get flags() { return this.regExp.flags; }
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
const testCases = [
    { regExp: /a/,  lastIndex: 0, input: "a", result: 0 },
    { regExp: /a/g, lastIndex: 0, input: "a", result: 0 },
    { regExp: /a/y, lastIndex: 0, input: "a", result: 1 },

    { regExp: /a/,  lastIndex: 0, input: "b", result: 0 },
    { regExp: /a/g, lastIndex: 0, input: "b", result: 0 },
    { regExp: /a/y, lastIndex: 0, input: "b", result: 0 },

    { regExp: /a/,  lastIndex: -0, input: "a", result: -0 },
    { regExp: /a/g, lastIndex: -0, input: "a", result: 0 },
    { regExp: /a/y, lastIndex: -0, input: "a", result: 1 },

    { regExp: /a/,  lastIndex: -0, input: "b", result: -0 },
    { regExp: /a/g, lastIndex: -0, input: "b", result: 0 },
    { regExp: /a/y, lastIndex: -0, input: "b", result: 0 },

    { regExp: /a/,  lastIndex: -1, input: "a", result: -1 },
    { regExp: /a/g, lastIndex: -1, input: "a", result: 0 },
    { regExp: /a/y, lastIndex: -1, input: "a", result: 1 },

    { regExp: /a/,  lastIndex: -1, input: "b", result: -1 },
    { regExp: /a/g, lastIndex: -1, input: "b", result: 0 },
    { regExp: /a/y, lastIndex: -1, input: "b", result: 0 },

    { regExp: /a/,  lastIndex: 100, input: "a", result: 100 },
    { regExp: /a/g, lastIndex: 100, input: "a", result: 0 },
    { regExp: /a/y, lastIndex: 100, input: "a", result: 0 },
];

for (let method of [RegExp.prototype[Symbol.match], RegExp.prototype[Symbol.replace]]) {
    for (let Constructor of [RegExp, DuckRegExp]) {
        // Basic test.
        for (let {regExp, lastIndex, input, result} of testCases) {
            let re = new Constructor(regExp);
            re.lastIndex = lastIndex;
            Reflect.apply(method, re, [input]);
            assert.sameValue(re.lastIndex, result);
        }

        // Test when lastIndex is non-writable.
        for (let {regExp, lastIndex, input} of testCases) {
            let re = new Constructor(regExp);
            Object.defineProperty(re, "lastIndex", { value: lastIndex, writable: false });
            if (re.global || re.sticky) {
                assert.throws(TypeError, () => Reflect.apply(method, re, [input]));
            } else {
                Reflect.apply(method, re, [input]);
            }
            assert.sameValue(re.lastIndex, lastIndex);
        }

        // Test when lastIndex is changed to non-writable as a side-effect.
        for (let {regExp, lastIndex, input, result} of testCases) {
            let re = new Constructor(regExp);
            let called = false;
            re.lastIndex = {
                valueOf() {
                    assert.sameValue(called, false);
                    called = true;
                    Object.defineProperty(re, "lastIndex", { value: 9000, writable: false });
                    return lastIndex;
                }
            };
            if (re.sticky) {
                assert.throws(TypeError, () => Reflect.apply(method, re, [input]));
                assert.sameValue(called, true);
                assert.sameValue(re.lastIndex, 9000);
            } else if (re.global) {
                Reflect.apply(method, re, [input]);
                assert.sameValue(called, false);
                assert.sameValue(re.lastIndex, result);
            } else {
                Reflect.apply(method, re, [input]);
                assert.sameValue(called, true);
                assert.sameValue(re.lastIndex, 9000);
            }
        }
    }
}

