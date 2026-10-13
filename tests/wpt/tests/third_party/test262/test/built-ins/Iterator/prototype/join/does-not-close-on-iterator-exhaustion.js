// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join does not close its receiver if the iterator is exhausted.
features: [Iterator.prototype.join]
---*/

var calledNextCount = 0;
var gotReturn = false;
var it = {
  next: function () {
    ++calledNextCount;
    if (calledNextCount > 2) {
      return { done: true, value: undefined };
    }
    return { done: false, value: 'ES' };
  },
  get return() {
    gotReturn = true;
  },
};

assert.sameValue(Iterator.prototype.join.call(it), 'ES,ES');

assert.sameValue(calledNextCount, 3);

assert.sameValue(gotReturn, false);
