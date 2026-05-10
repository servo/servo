// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.try forwards arguments
esid: sec-promise.try
features: [promise-try]
flags: [async]
includes: [asyncHelpers.js, compareArray.js]
---*/

var sentinel = { sentinel: true };

asyncTest(function () {
  return Promise.try(function () {
    assert.compareArray(
      Array.prototype.slice.call(arguments),
      [1, 2, Test262Error, sentinel]
    );
  }, 1, 2, Test262Error, sentinel)
});

