// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Verify an undefined descriptor
includes: [propertyHelper.js]
---*/
var sample = {
  bar: undefined,
  get baz() {}
};

assert.sameValue(
  verifyProperty(sample, "foo", undefined),
  true,
  "returns true if desc and property descriptor are both undefined"
);

assert.throws(Test262Error, () => {
  verifyProperty(sample, 'bar', undefined);
}, "dataDescriptor value is undefined");

assert.throws(Test262Error, () => {
  verifyProperty(sample, 'baz', undefined);
}, "accessor returns undefined");
