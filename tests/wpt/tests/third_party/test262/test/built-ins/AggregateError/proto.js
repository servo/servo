// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The prototype of AggregateError constructor is Error
esid: sec-aggregate-error
info: |
  Properties of the AggregateError Constructor

  - has a [[Prototype]] internal slot whose value is the intrinsic object %Error%.
features: [AggregateError]
---*/

var proto = Object.getPrototypeOf(AggregateError);

assert.sameValue(proto, Error);
