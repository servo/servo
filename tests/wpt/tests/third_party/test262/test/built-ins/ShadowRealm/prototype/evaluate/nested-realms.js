// Copyright (C) 2021 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm can create a nested ShadowRealm
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

globalThis.myValue = 'a';
const realm1 = new ShadowRealm();

realm1.evaluate('globalThis.myValue = "b";');

const realm2Evaluate = realm1.evaluate(`
  const realm2 = new ShadowRealm();

  (str) => realm2.evaluate(str);
`);

realm2Evaluate('globalThis.myValue = "c";');

assert.sameValue(globalThis.myValue, 'a');
assert.sameValue(realm1.evaluate('globalThis.myValue'), 'b');
assert.sameValue(realm2Evaluate('globalThis.myValue'), 'c');

realm1.evaluate('globalThis.myValue = "d"');

assert.sameValue(globalThis.myValue, 'a', 'no side effects');
assert.sameValue(realm1.evaluate('globalThis.myValue'), 'd', 'no side effects');
assert.sameValue(realm2Evaluate('globalThis.myValue'), 'c', 'no side effects');
