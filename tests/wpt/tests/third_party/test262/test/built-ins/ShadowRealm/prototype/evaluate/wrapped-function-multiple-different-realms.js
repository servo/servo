// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm can wrap a function to multiple realms.
features: [ShadowRealm]
---*/

const realm1 = new ShadowRealm();
const realm2 = new ShadowRealm();

globalThis.count = 0;

assert.notSameValue(realm1, realm2);

const r1wrapped = realm1.evaluate(`
  globalThis.count = 0;
  () => globalThis.count += 1;
`);

const r2wrapper = realm2.evaluate(`
  globalThis.wrapped = undefined;
  globalThis.count = 0; // Bait
  (fn) => globalThis.wrapped = fn;
`);

const rewrapped = r2wrapper(r1wrapped);

assert.notSameValue(rewrapped, r1wrapped, 'rewrapped !== r1wrapped');

const r2wrapped = realm2.evaluate('globalThis.wrapped');

assert.notSameValue(r2wrapped, r1wrapped, 'r2wrapped !== r1wrapped');
assert.notSameValue(r2wrapped, rewrapped, 'r2wrapped !== rewrapped');

assert.sameValue(realm1.evaluate('globalThis.count'), 0, `getting wrapped function won't trigger a call`);

assert.sameValue(r2wrapped(), 1, 'call from r2 wrapped (r2wrapped) cycles back to r1');

assert.sameValue(realm1.evaluate('globalThis.count'), 1, 'effects produced in a third realm, #1');

assert.sameValue(rewrapped(), 2, 'call from r2 wrapped (rewrapped) cycles back to r1');

assert.sameValue(realm1.evaluate('globalThis.count'), 2, 'effects produced in a third realm, #2');

assert.sameValue(realm2.evaluate('globalThis.count'), 0, 'no side effects produced in the wrong realm (realm2)');
assert.sameValue(globalThis.count, 0, 'no side effects produced in the wrong realm (main realm)');
