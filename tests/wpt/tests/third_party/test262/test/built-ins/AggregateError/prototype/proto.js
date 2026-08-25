// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-aggregate-error-prototype-objects
description: The prototype of AggregateError.prototype constructor is Error.prototype
info: |
  Properties of the AggregateError Prototype Object

  - has a [[Prototype]] internal slot whose value is the intrinsic object %Error.prototype%.
features: [AggregateError]
---*/

var proto = Object.getPrototypeOf(AggregateError.prototype);

assert.sameValue(proto, Error.prototype);
