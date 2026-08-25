// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.try errors when the receiver is not an object
esid: sec-promise.try
features: [promise-try]
---*/

assert.throws(TypeError, function () {
  Promise.try.call(undefined);
});

assert.throws(TypeError, function () {
  Promise.try.call(null);
});

assert.throws(TypeError, function () {
  Promise.try.call(86);
});

assert.throws(TypeError, function () {
  Promise.try.call('string');
});

assert.throws(TypeError, function () {
  Promise.try.call(true);
});

assert.throws(TypeError, function () {
  Promise.try.call(Symbol());
});
