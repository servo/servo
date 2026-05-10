// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 11.2.2_L15
description: >
    Tests that Intl.NumberFormat.supportedLocalesOf meets the
    requirements for built-in objects defined by the introduction of
    chapter 17 of the ECMAScript Language Specification.
author: Norbert Lindenberg
includes: [isConstructor.js]
features: [Reflect.construct]
---*/

assert.sameValue(Object.prototype.toString.call(Intl.NumberFormat.supportedLocalesOf), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(Intl.NumberFormat.supportedLocalesOf),
       "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.NumberFormat.supportedLocalesOf), Function.prototype);

assert.sameValue(Intl.NumberFormat.supportedLocalesOf.hasOwnProperty("prototype"), false,
                 "Built-in functions that aren't constructors must not have a prototype property.");

assert.sameValue(isConstructor(Intl.NumberFormat.supportedLocalesOf), false,
                 "Built-in functions don't implement [[Construct]] unless explicitly specified.");
