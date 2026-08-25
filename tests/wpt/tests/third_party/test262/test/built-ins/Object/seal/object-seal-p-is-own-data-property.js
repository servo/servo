// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setintegritylevel
description: Object.seal - 'P' is own data property
includes: [propertyHelper.js]
---*/

var obj = {};

obj.foo = 10; // default [[Configurable]] attribute value of foo: true

assert(Object.isExtensible(obj));
Object.seal(obj);

verifyProperty(obj, "foo", {
  value: 10,
  configurable: false,
});
