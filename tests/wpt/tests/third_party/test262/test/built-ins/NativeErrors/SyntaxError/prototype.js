// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.5.6.2.1
description: >
  The initial value of SyntaxError.prototype is the SyntaxError prototype object.
info: |
  The initial value of NativeError.prototype is a NativeError prototype object (19.5.6.3).
  Each NativeError constructor has a distinct prototype object.
  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

assert.sameValue(SyntaxError.prototype, Object.getPrototypeOf(new SyntaxError));

verifyProperty(SyntaxError, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
