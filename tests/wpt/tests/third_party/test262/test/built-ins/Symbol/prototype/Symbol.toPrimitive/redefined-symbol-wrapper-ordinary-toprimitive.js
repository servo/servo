// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.prototype-@@toprimitive
description: >
    If redefined to nullish value, Symbol wrapper object is converted to primitive
    via OrdinaryToPrimitive.
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

Object.defineProperty(Symbol.prototype, Symbol.toPrimitive, { value: null });

assert.sameValue(Object(Symbol()) == "Symbol()", false, "hint: default");
assert.throws(TypeError, () => { +Object(Symbol()); }, "hint: number");
assert.sameValue(`${Object(Symbol())}`, "Symbol()", "hint: string");

Object.defineProperty(Symbol.prototype, Symbol.toPrimitive, { value: undefined });

assert(Object(Symbol.iterator) == Symbol.iterator, "hint: default");
assert.throws(TypeError, () => { Object(Symbol()) <= ""; }, "hint: number");
assert.sameValue({ "Symbol()": 1 }[Object(Symbol())], 1, "hint: string");
