// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 10.3.2_L15
description: >
    Tests that the getter for Intl.Collator.prototype.compare meets
    the requirements for built-in objects defined by the introduction
    of chapter 17 of the ECMAScript Language Specification.
author: Norbert Lindenberg
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

var compareFn = Object.getOwnPropertyDescriptor(Intl.Collator.prototype, "compare").get;

assert.sameValue(Object.prototype.toString.call(compareFn), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(compareFn),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(compareFn), Function.prototype);

assert.sameValue(compareFn.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(compareFn), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
