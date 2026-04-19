// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.prototype-@@toprimitive
description: >
    If deleted, Symbol wrapper objects is converted to primitive via OrdinaryToPrimitive.
info: |
    ToPrimitive ( input [ , preferredType ] )

    [...]
    2. If Type(input) is Object, then
        a. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
        b. If exoticToPrim is not undefined, then
            [...]
        c. If preferredType is not present, let preferredType be number.
        d. Return ? OrdinaryToPrimitive(input, preferredType).
features: [Symbol.toPrimitive]
---*/

assert(delete Symbol.prototype[Symbol.toPrimitive]);

let valueOfGets = 0;
let valueOfCalls = 0;
let valueOfFunction = () => { ++valueOfCalls; return 123; };
Object.defineProperty(Symbol.prototype, "valueOf", {
    get: () => { ++valueOfGets; return valueOfFunction; },
});

assert(Object(Symbol()) == 123, "hint: default");
assert.sameValue(Object(Symbol()) - 0, 123, "hint: number");
assert.sameValue("".concat(Object(Symbol())), "Symbol()", "hint: string");

assert.sameValue(valueOfGets, 2);
assert.sameValue(valueOfCalls, 2);

let toStringGets = 0;
let toStringCalls = 0;
let toStringFunction = () => { ++toStringCalls; return "foo"; };
Object.defineProperty(Symbol.prototype, "toString", {
    get: () => { ++toStringGets; return toStringFunction; },
});

assert.sameValue("" + Object(Symbol()), "123", "hint: default");
assert.sameValue(Object(Symbol()) * 1, 123, "hint: number");
assert.sameValue({ "123": 1, "Symbol()": 2, "foo": 3 }[Object(Symbol())], 3, "hint: string");

assert.sameValue(valueOfGets, 4);
assert.sameValue(valueOfCalls, 4);
assert.sameValue(toStringGets, 1);
assert.sameValue(toStringCalls, 1);

valueOfFunction = null;

assert.sameValue(new Date(Object(Symbol())).getTime(), NaN, "hint: default");
assert.sameValue(+Object(Symbol()), NaN, "hint: number");
assert.sameValue(`${Object(Symbol())}`, "foo", "hint: string");

assert.sameValue(valueOfGets, 6);
assert.sameValue(valueOfCalls, 4);
assert.sameValue(toStringGets, 4);
assert.sameValue(toStringCalls, 4);

toStringFunction = function() { throw new Test262Error(); };

assert.throws(Test262Error, () => { Object(Symbol()) != 123; }, "hint: default");
assert.throws(Test262Error, () => { Object(Symbol()) / 0; }, "hint: number");
assert.throws(Test262Error, () => { "".concat(Object(Symbol())); }, "hint: string");

assert.sameValue(valueOfGets, 8);
assert.sameValue(valueOfCalls, 4);
assert.sameValue(toStringGets, 7);
assert.sameValue(toStringCalls, 4);

toStringFunction = undefined;

assert.throws(TypeError, () => { 1 + Object(Symbol()); }, "hint: default");
assert.throws(TypeError, () => { Number(Object(Symbol())); }, "hint: number");
assert.throws(TypeError, () => { String(Object(Symbol())); }, "hint: string");

assert.sameValue(valueOfGets, 11);
assert.sameValue(valueOfCalls, 4);
assert.sameValue(toStringGets, 10);
assert.sameValue(toStringCalls, 4);
