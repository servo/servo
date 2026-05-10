// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 26.2.2.1.1
description: The `name` property of Proxy Revocation functions
info: |
  A Proxy revocation function is an anonymous function.

  17 ECMAScript Standard Built-in Objects:
    Every built-in function object, including constructors, has a `name`
    property whose value is a String. Functions that are identified as
    anonymous functions use the empty string as the value of the `name`
    property.
    Unless otherwise specified, the `name` property of a built-in function
    object has the attributes { [[Writable]]: *false*, [[Enumerable]]: *false*,
    [[Configurable]]: *true* }.
includes: [propertyHelper.js]
features: [Proxy]
---*/

var revocationFunction = Proxy.revocable({}, {}).revoke;

verifyProperty(revocationFunction, "name", {
  value: "", writable: false, enumerable: false, configurable: true
});
