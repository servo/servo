// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: pending
description: |
  %Iterator.prototype%.map accesses specified properties only.
features:
  - iterator-helpers
---*/
const handlerProxy = log => new Proxy({}, {
  get: (target, key, receiver) => (...args) => {
    const target = args[0];
    const item = Reflect[key](...args);

    log.push(`${key}: ${args.filter(x => typeof x != 'object').map(x => x.toString())}`);

    switch (typeof item) {
      case 'function': return item.bind(new Proxy(target, handlerProxy(log)));
      case 'object': return new Proxy(item, handlerProxy(log));
      default: return item;
    }
  },
});

const log = [];
const iterator = Object.setPrototypeOf({
  next: function() {
    if (this.value < 3)
      return { done: false, value: this.value++ };
    return { done: true, value: undefined };
  },
  value: 0,
}, Iterator.prototype);
const iteratorProxy = new Proxy(iterator, handlerProxy(log));
const mappedProxy = iteratorProxy.map(x => x);

for (const item of mappedProxy) {
}

assert.sameValue(
  log.join('\n'),
`get: map
get: next
get: value
get: value
getOwnPropertyDescriptor: value
has: enumerable
get: enumerable
has: configurable
get: configurable
has: value
get: value
has: writable
get: writable
has: get
has: set
defineProperty: value
set: value,1
get: value
get: value
getOwnPropertyDescriptor: value
has: enumerable
get: enumerable
has: configurable
get: configurable
has: value
get: value
has: writable
get: writable
has: get
has: set
defineProperty: value
set: value,2
get: value
get: value
getOwnPropertyDescriptor: value
has: enumerable
get: enumerable
has: configurable
get: configurable
has: value
get: value
has: writable
get: writable
has: get
has: set
defineProperty: value
set: value,3
get: value`
);

