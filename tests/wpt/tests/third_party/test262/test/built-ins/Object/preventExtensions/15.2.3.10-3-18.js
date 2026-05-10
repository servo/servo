// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-3-18
description: >
    Object.preventExtensions - named properties cannot be added into a
    Date object
includes: [propertyHelper.js]
---*/

var obj = new Date(0);

assert(Object.isExtensible(obj));
Object.preventExtensions(obj);
assert(!Object.isExtensible(obj));

verifyNotWritable(obj, "exName", "nocheck");

assert(!obj.hasOwnProperty("exName"));
