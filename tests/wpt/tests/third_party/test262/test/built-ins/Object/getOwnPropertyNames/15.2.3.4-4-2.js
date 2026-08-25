// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-2
description: Object.getOwnPropertyNames returns array of property names (Object)
---*/

var result = Object.getOwnPropertyNames(Object);

assert(result.indexOf("getPrototypeOf") > -1, "getPrototypeOf");
assert(result.indexOf("getOwnPropertyDescriptor") > -1, "getOwnPropertyDescriptor");
assert(result.indexOf("getOwnPropertyNames") > -1, "getOwnPropertyNames");
assert(result.indexOf("create") > -1, "create");
assert(result.indexOf("defineProperty") > -1, "defineProperty");
assert(result.indexOf("defineProperties") > -1, "defineProperties");
assert(result.indexOf("seal") > -1, "seal");
assert(result.indexOf("freeze") > -1, "freeze");
assert(result.indexOf("preventExtensions") > -1, "preventExtensions");
assert(result.indexOf("isSealed") > -1, "isSealed");
assert(result.indexOf("isFrozen") > -1, "isFrozen");
assert(result.indexOf("isExtensible") > -1, "isExtensible");
assert(result.indexOf("keys") > -1, "keys");
assert(result.indexOf("prototype") > -1, "prototype");
assert(result.indexOf("length") > -1, "length");
