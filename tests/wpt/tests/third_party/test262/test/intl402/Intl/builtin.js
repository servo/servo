// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: intl-object
description: >
    Tests that Intl meets the requirements for built-in objects
    defined by the introduction of chapter 17 of the ECMAScript
    Language Specification.
author: Norbert Lindenberg
---*/

assert(Object.isExtensible(Intl), "Built-in objects must be extensible.");

assert.sameValue(Object.getPrototypeOf(Intl), Object.prototype,
                 "The [[Prototype]] of Intl is %ObjectPrototype%.");

assert.sameValue(this.Intl, Intl,
                 "%Intl% is accessible as a property of the global object.");
