// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error.prototype.message
description: >
  The `AggregateError.prototype.message` property descriptor.
info: |
  The initial value of the message property of the prototype for a given AggregateError
  constructor is the empty String.

  17 ECMAScript Standard Built-in Objects:

  Every other data property described (...) has the attributes { [[Writable]]: true,
    [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [AggregateError]
---*/

verifyProperty(AggregateError.prototype, 'message', {
  value: '',
  enumerable: false,
  writable: true,
  configurable: true
});
