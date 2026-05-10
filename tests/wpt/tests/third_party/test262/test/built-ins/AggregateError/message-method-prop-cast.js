// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  Cast ToString values of message in the created method property
info: |
  AggregateError ( errors, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.

  CreateMethodProperty ( O, P, V )

  ...
  3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }.
  4. Return ? O.[[DefineOwnProperty]](P, newDesc).
features: [AggregateError]
includes: [propertyHelper.js]
---*/

var case1 = new AggregateError([], 42);

verifyProperty(case1, 'message', {
  value: '42',
  writable: true,
  enumerable: false,
  configurable: true,
});

var case2 = new AggregateError([], false);

verifyProperty(case2, 'message', {
  value: 'false',
  writable: true,
  enumerable: false,
  configurable: true,
});

var case3 = new AggregateError([], true);

verifyProperty(case3, 'message', {
  value: 'true',
  writable: true,
  enumerable: false,
  configurable: true,
});

var case4 = new AggregateError([], { toString() { return 'string'; }});

verifyProperty(case4, 'message', {
  value: 'string',
  writable: true,
  enumerable: false,
  configurable: true,
});

var case5 = new AggregateError([], null);

verifyProperty(case5, 'message', {
  value: 'null',
  writable: true,
  enumerable: false,
  configurable: true,
});
