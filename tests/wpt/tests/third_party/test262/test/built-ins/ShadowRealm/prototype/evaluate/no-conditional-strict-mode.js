// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  The new realm has no conditional strict mode based on its outer realm
info: |
  This test should always run with the outer realm in both strict and non
  strict mode to verify the realm code starts in non-strict mode.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

const res = r.evaluate(`
  function lol() {
    arguments = 42; // This would be a SyntaxError if in strict mode

    return arguments;
  }
  lol;
`);

assert.sameValue(res(), 42);

const res2 = r.evaluate('var public = 1; 42');

assert.sameValue(res2, 42);
