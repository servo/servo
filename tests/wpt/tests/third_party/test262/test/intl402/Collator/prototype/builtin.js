// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 10.3_L15
description: >
    Tests that Intl.Collator.prototype meets the requirements for
    built-in objects defined by the introduction of chapter 17 of the
    ECMAScript Language Specification.
author: Norbert Lindenberg
---*/

assert(Object.isExtensible(Intl.Collator.prototype), "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl.Collator.prototype), Object.prototype,
                 "Built-in prototype objects must have Object.prototype as their prototype.");
