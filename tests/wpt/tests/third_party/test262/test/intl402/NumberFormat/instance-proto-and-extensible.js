// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.3
description: >
    Tests that objects constructed by Intl.NumberFormat have the
    specified internal properties.
author: Norbert Lindenberg
---*/

var obj = new Intl.NumberFormat();

var actualPrototype = Object.getPrototypeOf(obj);
assert.sameValue(actualPrototype, Intl.NumberFormat.prototype, "Prototype of object constructed by Intl.NumberFormat isn't Intl.NumberFormat.prototype.");

assert(Object.isExtensible(obj), "Object constructed by Intl.NumberFormat must be extensible.");
