// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 26.2.2.1.1
description: The `length` property of Proxy Revocation functions
info: |
  The length property of a Proxy revocation function is 0.

  17 ECMAScript Standard Built-in Objects:
    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Proxy]
---*/

var revocationFunction = Proxy.revocable({}, {}).revoke;

verifyProperty(revocationFunction, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
