// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The first argument should have an own property
includes: [propertyHelper.js]
---*/
assert.throws(Test262Error, () => {
  verifyProperty(Object, 'JeanPaulSartre', {});
}, "inexisting property");

assert.throws(Test262Error, () => {
  verifyProperty({}, 'hasOwnProperty', {});
}, "inexisting own property");
