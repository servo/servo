// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.map passes lastValue to the `next` call.
info: |
  Iterator Helpers Proposal 2.1.5.2
features:
  - iterator-helpers
---*/
const iteratorWhereNextTakesValue = Object.setPrototypeOf({
  next: function(value) {
    assert.sameValue(arguments.length, 0);

    if (this.value < 3)
      return { done: false, value: this.value++ };
    return { done: true, value: undefined };
  },
  value: 0,
}, Iterator.prototype);

const mappedIterator = iteratorWhereNextTakesValue.map(x => x);

assert.sameValue(mappedIterator.next(1).value, 0);

assert.sameValue(mappedIterator.next(2).value, 1);

assert.sameValue(mappedIterator.next(3).value, 2);

assert.sameValue(mappedIterator.next(4).done, true);

assert.sameValue(mappedIterator.next(5).done, true);

