// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack
description: AsyncDisposableStack.name property descriptor
info: |
  AsyncDisposableStack ( )

  17 ECMAScript Standard Built-in Objects

  Every built-in function object, including constructors, that is not
  identified as an anonymous function has a name property whose value
  is a String. Unless otherwise specified, this value is the name that
  is given to the function in this specification. For functions that
  are specified as properties of objects, the name value is the
  property name string used to access the function. [...]

  Unless otherwise specified, the name property of a built-in function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [explicit-resource-management]
---*/

verifyProperty(AsyncDisposableStack, 'name', {
  value: 'AsyncDisposableStack',
  writable: false,
  enumerable: false,
  configurable: true
});
