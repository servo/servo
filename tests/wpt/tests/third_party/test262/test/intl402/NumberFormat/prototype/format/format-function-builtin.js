// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 11.3.2_1_a_L15
description: >
    Tests that the function returned by
    Intl.NumberFormat.prototype.format meets the requirements for
    built-in objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Norbert Lindenberg
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

var formatFn = new Intl.NumberFormat().format;

assert.sameValue(Object.prototype.toString.call(formatFn), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(formatFn),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(formatFn), Function.prototype);

assert.sameValue(formatFn.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(formatFn), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
