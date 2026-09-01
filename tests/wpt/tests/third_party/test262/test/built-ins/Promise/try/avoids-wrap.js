// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: When the callback passed to Promise.try returns a Promise, it is not wrapped
esid: sec-promise.try
features: [promise-try]
---*/

var sentinel = Promise.resolve();

var returnValue = Promise.try(function () {
  return sentinel;
});
assert.sameValue(returnValue, sentinel);
