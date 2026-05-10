// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
author: Jordan Harband
description: Promise subclass finally on resolved creates the proper number of subclassed promises
esid: sec-promise.prototype.finally
features: [Promise.prototype.finally]
flags: [async]
---*/

var count = 0;
class FooPromise extends Promise {
  constructor(resolve, reject) {
    count++;
    return super(resolve, reject);
  }
}

FooPromise.resolve().finally(() => {}).then(() => {
  assert.sameValue(count, 7);
}).then($DONE, $DONE);
