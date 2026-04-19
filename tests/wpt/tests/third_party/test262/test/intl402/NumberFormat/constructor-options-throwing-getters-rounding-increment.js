// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializenumberformat
description: >
  Exception from accessing the "roundingIncrement" option for the NumberFormat
  constructor should be propagated to the caller
features: [Intl.NumberFormat-v3]
---*/

assert.throws(Test262Error, function() {
  new Intl.NumberFormat('en', {
    get roundingIncrement() {
      throw new Test262Error();
    }
  });
});
