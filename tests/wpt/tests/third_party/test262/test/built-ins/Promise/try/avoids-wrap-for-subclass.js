// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: When the callback passed to PromiseSubclass.try returns a PromiseSubclass, it is not wrapped
esid: sec-promise.try
features: [promise-try, class]
---*/

class SubPromise extends Promise {
  constructor(executor) {
    super(executor);
  }
}

var sentinel = SubPromise.resolve();
assert(sentinel instanceof SubPromise);

var returnValue = SubPromise.try(function () {
  return sentinel;
});
assert.sameValue(returnValue, sentinel);
