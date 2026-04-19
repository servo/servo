// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 13.3.3_L15
description: >
    Tests that Date.prototype.toLocaleTimeString meets the
    requirements for built-in objects defined by the introduction of
    chapter 17 of the ECMAScript Language Specification.
author: Norbert Lindenberg
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

assert.sameValue(Object.prototype.toString.call(Date.prototype.toLocaleTimeString), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(Date.prototype.toLocaleTimeString),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Date.prototype.toLocaleTimeString), Function.prototype);

assert.sameValue(Date.prototype.toLocaleTimeString.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(Date.prototype.toLocaleTimeString), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
