// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.3
description: >
    Tests that objects constructed by Intl.Collator have the specified
    internal properties.
author: Norbert Lindenberg
---*/

var obj = new Intl.Collator();

var actualPrototype = Object.getPrototypeOf(obj);
assert.sameValue(actualPrototype, Intl.Collator.prototype, "Prototype of object constructed by Intl.Collator isn't Intl.Collator.prototype.");

assert(Object.isExtensible(obj), "Object constructed by Intl.Collator must be extensible.");
