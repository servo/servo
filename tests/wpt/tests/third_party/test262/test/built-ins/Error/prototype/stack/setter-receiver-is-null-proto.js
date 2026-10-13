// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  The setter installs an own "stack" data property on a null-prototype object.
  The setter does not depend on inherited machinery from Object.prototype.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.
includes: [propertyHelper.js]
features: [error-stack-accessor, __proto__]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

var nullProto = { __proto__: null };
set.call(nullProto, 'null-proto');

verifyProperty(nullProto, 'stack', {
  value: 'null-proto',
  writable: true,
  enumerable: true,
  configurable: true,
});
