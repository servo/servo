// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped function arguments are wrapped into the inner realm, extended.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();
const blueFn = (x, y) => x + y;

const redWrappedFn = r.evaluate(`
    function fn(wrapped1, wrapped2, wrapped3) {
        if (wrapped1.x) {
            return 1;
        }
        if (wrapped2.x) {
            return 2;
        }
        if (wrapped3.x) {
            // Not unwrapped
            return 3;
        }
        if (wrapped1 === wrapped2) {
            // Always a new wrapped function
            return 4;
        }

        // No unwrapping
        if (wrapped3 === fn) {
            return 5;
        };

        return true;
    }
    fn.x = 'secret';
    fn;
`);
assert.sameValue(redWrappedFn(blueFn, blueFn, redWrappedFn), true);
