// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
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
      return item.bind(receiver);
    return item;
  },
});
const iter = new Proxy({
  next: () => ({ done: false, value: 0 }),
}, handlerProxy);

const wrap = Iterator.from(iter);
// Call next multiple times. Should not call `get` on proxy.
wrap.next();
wrap.next();
wrap.next();

assert.compareArray(log, [
  "get: Symbol(Symbol.iterator)",
  "get: next",
  "getPrototypeOf: undefined",
]);

