// Copyright 2012 Mozilla Corporation. All rights reserved.
// Copyright 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tolocalestring
description: >
    Tests that BigInt.prototype.toLocaleString meets the requirements
    for built-in objects defined by the introduction of chapter 17 of
    the ECMAScript Language Specification.
author: Norbert Lindenberg
includes: [isConstructor.js]
features: [Reflect.construct, BigInt]
---*/

assert.sameValue(Object.prototype.toString.call(BigInt.prototype.toLocaleString), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(BigInt.prototype.toLocaleString),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(BigInt.prototype.toLocaleString), Function.prototype);

assert.sameValue(BigInt.prototype.toLocaleString.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(BigInt.prototype.toLocaleString), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
