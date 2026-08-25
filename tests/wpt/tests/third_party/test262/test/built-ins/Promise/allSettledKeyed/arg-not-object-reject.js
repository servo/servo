// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Promise.allSettledKeyed rejects when the promises argument is not an Object
info: |
  Promise.allSettledKeyed ( promises )

  ...
  5. If promises is not an Object, then
    a. Let error be a newly created TypeError object.
    b. Perform ? Call(promiseCapability.[[Reject]], undefined, « error »).
    c. Return promiseCapability.[[Promise]].
includes: [asyncHelpers.js]
flags: [async]
features: [await-dictionary, Symbol]
---*/

asyncTest(function() {
  function check(value, msg) {
    return assert.throwsAsync(TypeError, function() {
      return Promise.allSettledKeyed(value);
    }, msg);
  }

  return check(undefined, 'undefined')
    .then(function() { return check(null, 'null'); })
    .then(function() { return check(86, 'number'); })
    .then(function() { return check('string', 'string'); })
    .then(function() { return check(true, 'boolean'); })
    .then(function() { return check(Symbol(), 'symbol'); });
});
