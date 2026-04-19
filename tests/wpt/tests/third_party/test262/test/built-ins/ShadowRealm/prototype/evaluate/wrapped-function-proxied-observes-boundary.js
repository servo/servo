// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  Proxying a wrapped function and invoking it still performs boundary checks
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const wrapped = r.evaluate(`() => { return 1; };`);

const secretObj = {x: 2};

let received;

const proxiedWrapped = new Proxy(wrapped, {
  apply(target, _, args) {
    assert.sameValue(target, wrapped);
    received = args;

    // Object can't be sent to the other Realm
    return target({x: 1});
  }
});

assert.throws(
  TypeError,
  () => proxiedWrapped(secretObj),
  'Proxying a wrapped function and invoking it still performs boundary checks'
);

assert.sameValue(received[0], secretObj, 'proxy still calls the handler trap');
assert.sameValue(received.length, 1);
