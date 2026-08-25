// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.supportedvaluesof
description: >
  Intl.supportedValuesOf is a built-in function object..
info: |
  Intl.supportedValuesOf ( key )

  18 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, a built-in object that is callable as a function
    is a built-in function object with the characteristics described in 10.3.
    Unless specified otherwise, the [[Extensible]] internal slot of a built-in
    object initially has the value true.

    Unless otherwise specified every built-in function and every built-in
    constructor has the Function prototype object, which is the initial value
    of the expression Function.prototype (20.2.3), as the value of its
    [[Prototype]] internal slot.

    Built-in function objects that are not identified as constructors do not
    implement the [[Construct]] internal method unless otherwise specified in
    the description of a particular function.
includes: [isConstructor.js]
features: [Intl-enumeration, Reflect.construct]
---*/

assert.sameValue(typeof Intl.supportedValuesOf, "function",
                 "Intl.supportedValuesOf is a function");

assert(!Object.prototype.hasOwnProperty.call(Intl.supportedValuesOf, "prototype"),
       "Intl.supportedValuesOf doesn't have an own 'prototype' property");

assert(Object.isExtensible(Intl.supportedValuesOf),
       "Built-in objects must be extensible");

assert.sameValue(Object.getPrototypeOf(Intl.supportedValuesOf), Function.prototype,
                 "[[Prototype]] of Intl.supportedValuesOf is Function.prototype");

assert(!isConstructor(Intl.supportedValuesOf),
       "Intl.supportedValuesOf not a constructor function");
