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
    return { done: false, value: 0 };
  },
  return: function(value) {
    log.push('close iterator');
    return { done: true, value };
  },
}, Iterator.prototype);
const iteratorProxy = new Proxy(iterator, handlerProxy(log));
const mappedProxy = iteratorProxy.map(x => { throw 'error'; });

try {
  mappedProxy.next();
} catch (exc) {
  assert.sameValue(exc, 'error');
}

assert.sameValue(
  log.join('\n'),
`get: map
get: next
get: return
close iterator`
);

