// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 12.1_L15
description: >
    Tests that Intl.DateTimeFormat meets the requirements for
    built-in objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Norbert Lindenberg
---*/

assert.sameValue(Object.prototype.toString.call(Intl.DateTimeFormat), "[object Function]",
                 "The [[Class]] internal property of a built-in function must be " +
                 "\"Function\".");

assert(Object.isExtensible(Intl.DateTimeFormat), "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.DateTimeFormat), Function.prototype);
