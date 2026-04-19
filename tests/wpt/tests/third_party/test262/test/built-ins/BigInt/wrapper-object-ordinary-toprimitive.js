// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-toprimitive
description: >
    BigInt wrapper object is converted to primitive via OrdinaryToPrimitive.
info: |
    ToPrimitive ( input [ , preferredType ] )

    [...]
    2. If Type(input) is Object, then
        a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
        b. If exoticToPrim is not undefined, then
            [...]
        c. If preferredType is not present, let preferredType be number.
        d. Return ? OrdinaryToPrimitive(input, preferredType).
features: [BigInt]
---*/

const BigIntToString = BigInt.prototype.toString;
let toStringGets = 0;
let toStringCalls = 0;
let toStringFunction = function() { ++toStringCalls; return `${BigIntToString.call(this)}foo`; };
Object.defineProperty(BigInt.prototype, "toString", {
    get: () => { ++toStringGets; return toStringFunction; },
});

assert.sameValue("" + Object(1n), "1", "hint: default");
assert.throws(TypeError, () => { +Object(1n); }, "hint: number");
assert.sameValue(`${Object(1n)}`, "1foo", "hint: string");

assert.sameValue(toStringGets, 1);
assert.sameValue(toStringCalls, 1);

const BigIntValueOf = BigInt.prototype.valueOf;
let valueOfGets = 0;
let valueOfCalls = 0;
let valueOfFunction = function() { ++valueOfCalls; return BigIntValueOf.call(this) * 2n; };
Object.defineProperty(BigInt.prototype, "valueOf", {
    get: () => { ++valueOfGets; return valueOfFunction; },
});

assert(Object(1n) == 2n, "hint: default");
assert.sameValue(Object(1n) + 1n, 3n, "hint: number");
assert.sameValue({ "1foo": 1, "2": 2 }[Object(1n)], 1, "hint: string");

assert.sameValue(toStringGets, 2);
assert.sameValue(toStringCalls, 2);
assert.sameValue(valueOfGets, 2);
assert.sameValue(valueOfCalls, 2);

toStringFunction = undefined;

assert.throws(TypeError, () => { 1 + Object(1n); }, "hint: default");
assert.sameValue(Object(1n) * 1n, 2n, "hint: number");
assert.sameValue("".concat(Object(1n)), "2", "hint: string");

assert.sameValue(toStringGets, 3);
assert.sameValue(toStringCalls, 2);
assert.sameValue(valueOfGets, 5);
assert.sameValue(valueOfCalls, 5);

valueOfFunction = null;

assert.throws(TypeError, () => { new Date(Object(1n)); }, "hint: default");
assert.throws(TypeError, () => { Number(Object(1n)); }, "hint: number");
assert.throws(TypeError, () => { String(Object(1n)); }, "hint: string");

assert.sameValue(toStringGets, 6);
assert.sameValue(toStringCalls, 2);
assert.sameValue(valueOfGets, 8);
assert.sameValue(valueOfCalls, 5);
