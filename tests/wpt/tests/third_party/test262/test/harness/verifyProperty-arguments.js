// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  verifyProperty should receive at least 3 arguments: obj, name, and descriptor
includes: [propertyHelper.js]
---*/
assert.throws(Test262Error, () => {
  verifyProperty();
}, "0 arguments");

assert.throws(Test262Error, () => {
  verifyProperty(Object);
}, "1 argument");

assert.throws(Test262Error, () => {
  verifyProperty(Object, 'foo');
}, "2 arguments");
