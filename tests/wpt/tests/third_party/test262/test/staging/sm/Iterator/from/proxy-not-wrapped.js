// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - iterator-helpers
info: |
  Iterator is not enabled unconditionally
description: |
  pending
esid: pending
---*/
const log = [];
const handlerProxy = new Proxy({}, {
  get: (target, key, receiver) => (...args) => {
    log.push(`${key}: ${args[1]?.toString()}`);

    const item = Reflect[key](...args);
    if (typeof item === 'function')
      return (...args) => new Proxy(item.apply(receiver, args), handlerProxy);
    return item;
  },
});

class Iter extends Iterator {
  [Symbol.iterator]() {
    return this;
  }
  next() {
    return { done: false, value: 0 };
  }
}
const iter = new Iter();
const proxy = new Proxy(iter, handlerProxy);
const wrap = Iterator.from(proxy);

assert.sameValue(
  log.join('\n'),
  `get: Symbol(Symbol.iterator)
get: next
getPrototypeOf: undefined`
);

