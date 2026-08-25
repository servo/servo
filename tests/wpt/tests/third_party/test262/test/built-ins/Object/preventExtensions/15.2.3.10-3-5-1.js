// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.10-3-5-1
description: >
    Object.preventExtensions - indexed properties cannot be added into
    a String object
includes: [propertyHelper.js]
---*/

var obj = new String("bbq");

assert(Object.isExtensible(obj));
Object.preventExtensions(obj);
assert(!Object.isExtensible(obj));

verifyNotWritable(obj, "10", "nocheck");

assert(!obj.hasOwnProperty("10"));
