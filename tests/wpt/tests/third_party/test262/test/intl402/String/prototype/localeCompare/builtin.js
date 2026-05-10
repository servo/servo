// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 13.1.1_L15
description: >
    Tests that String.prototype.localeCompare meets the requirements
    for built-in objects defined by the introduction of chapter 17 of
    the ECMAScript Language Specification.
author: Norbert Lindenberg
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

assert.sameValue(Object.prototype.toString.call(String.prototype.localeCompare), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(String.prototype.localeCompare),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(String.prototype.localeCompare), Function.prototype);

assert.sameValue(String.prototype.localeCompare.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(String.prototype.localeCompare), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
