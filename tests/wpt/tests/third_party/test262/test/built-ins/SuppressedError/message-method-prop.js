// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  Creates a method property for message
info: |
  SuppressedError ( error, suppressed, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.

  CreateMethodProperty ( O, P, V )

  ...
  3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }.
  4. Return ? O.[[DefineOwnProperty]](P, newDesc).
features: [explicit-resource-management]
includes: [propertyHelper.js]
---*/

var obj = new SuppressedError(undefined, undefined, '42');

verifyProperty(obj, 'message', {
  value: '42',
  writable: true,
  enumerable: false,
  configurable: true,
});
