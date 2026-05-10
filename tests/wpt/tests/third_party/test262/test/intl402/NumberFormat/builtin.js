// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 11.1_L15
description: >
    Tests that Intl.NumberFormat meets the requirements for built-in
    objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Norbert Lindenberg
---*/

assert.sameValue(Object.prototype.toString.call(Intl.NumberFormat), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(Intl.NumberFormat), "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.NumberFormat), Function.prototype);
