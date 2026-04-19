// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-instances-length
description: >
  Async functions have a length property that is the number of expected
  arguments.
includes: [propertyHelper.js]
---*/

async function l0() {}
async function l1(a) {}
async function l2(a, b) {}
assert.sameValue(l0.length, 0);
assert.sameValue(l1.length, 1);
assert.sameValue(l2.length, 2)

verifyProperty(l0, 'length', {
  writable: false,
  enumerable: false,
  configurable: true,
});
