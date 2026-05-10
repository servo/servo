// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.try returns a Promise resolved to the callback's return value
esid: sec-promise.try
features: [promise-try]
flags: [async]
includes: [asyncHelpers.js]
---*/

var sentinel = { sentinel: true };

asyncTest(function() {
  return Promise.try(function () {
    return sentinel;
  }).then(function (v) {
    assert.sameValue(v, sentinel);
  })
});

