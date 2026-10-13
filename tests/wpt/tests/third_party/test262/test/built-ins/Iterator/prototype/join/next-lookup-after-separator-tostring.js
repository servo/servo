// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join looks up `next` on its receiver only after coercing the separator.
features: [Iterator.prototype.join]
---*/

var effects = [];

var separator = {
  toString: function () {
    effects.push('toString');
    return '&&';
  },
};

var calledNextCount = 0;
var it = {
  get next() {
    effects.push('get next');
    return function () {
      ++calledNextCount;
      if (calledNextCount === 1) {
        return { done: false, value: 'one' };
      } else if (calledNextCount === 2) {
        return { done: false, value: 'two' };
      } else {
        return { done: true, value: undefined };
      }
    };
  },
};

assert.sameValue(Iterator.prototype.join.call(it, separator), 'one&&two');
assert.sameValue(calledNextCount, 3);

assert.compareArray(effects, ['toString', 'get next']);
