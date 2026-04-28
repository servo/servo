// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped functions share no properties
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const wrapped = r.evaluate(`
function fn() {
    return fn.secret;
}

fn.secret = 'confidential';
fn;
`);

assert.sameValue(wrapped.secret, undefined);
assert.sameValue(wrapped(), 'confidential');
