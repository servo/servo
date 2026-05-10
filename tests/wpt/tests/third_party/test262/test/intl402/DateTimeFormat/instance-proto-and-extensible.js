// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.1.3
description: >
    Tests that objects constructed by Intl.DateTimeFormat have the
    specified internal properties.
author: Norbert Lindenberg
---*/

var obj = new Intl.DateTimeFormat();

var actualPrototype = Object.getPrototypeOf(obj);
assert.sameValue(actualPrototype, Intl.DateTimeFormat.prototype, "Prototype of object constructed by Intl.DateTimeFormat isn't Intl.DateTimeFormat.prototype.");

assert(Object.isExtensible(obj), "Object constructed by Intl.DateTimeFormat must be extensible.");
