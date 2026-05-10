// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped function observing their scopes
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();
let myValue;

function blueFn(x) {
    myValue = x;
    return myValue;
}

// cb is a new function in the red ShadowRealm that chains the call to the blueFn
const redFunction = r.evaluate(`
    var myValue = 'red';
    0, function(cb) {
        cb(42);
        return myValue;
    };
`);

assert.sameValue(redFunction(blueFn), 'red');
assert.sameValue(myValue, 42);
