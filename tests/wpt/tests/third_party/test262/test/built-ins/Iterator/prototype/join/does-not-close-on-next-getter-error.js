// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join does not close its receiver if looking up `next` throws.
features: [Iterator.prototype.join]
---*/

var gotReturn = false;
var it = {
  get next() {
    throw new Test262Error();
  },
  get return() {
    gotReturn = true;
  },
};

assert.throws(Test262Error, function () {
  Iterator.prototype.join.call(it);
});

assert.sameValue(gotReturn, false);
