// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.try produces instances of the receiver
esid: sec-promise.try
features: [promise-try, class]
---*/

var executor = null;
var callCount = 0;

class SubPromise extends Promise {
  constructor(a) {
    super(a);
    executor = a;
    callCount += 1;
  }
}

var instance = Promise.try.call(SubPromise, function () {});

assert.sameValue(instance.constructor, SubPromise);
assert.sameValue(instance instanceof SubPromise, true);

assert.sameValue(callCount, 1);
assert.sameValue(typeof executor, 'function');
