// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes supports a this value that does not inherit from
  Iterator.prototype but implements the iterator protocol
features: [iterator-includes]
---*/

let iter = {
  get next() {
    let count = 3;
    return function() {
      --count;
      return count >= 0 ? { done: false, value: count } : { done: true, value: undefined };
    };
  },
};

let result = Iterator.prototype.includes.call(iter, 0);

assert.sameValue(result, true);
