// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Promise.allSettledKeyed rejects when the promises argument is a BigInt
info: |
  Promise.allSettledKeyed ( promises )

  ...
  5. If promises is not an Object, then
    a. Let error be a newly created TypeError object.
    b. Perform ? Call(promiseCapability.[[Reject]], undefined, « error »).
    c. Return promiseCapability.[[Promise]].
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, BigInt]
---*/

asyncTest(function() {
  return assert.throwsAsync(TypeError, function() {
    return Promise.allSettledKeyed(0n);
  }, 'BigInt');
});
