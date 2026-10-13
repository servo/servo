// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  The home-object check uses SameValue on object identity. A Proxy wrapping
  %Error.prototype% is not the same object as %Error.prototype% itself, so
  step 2 of SetterThatIgnoresPrototypeProperties does not fire; the algorithm
  proceeds to consult the proxy's [[GetOwnProperty]] / [[Set]] traps.
info: |
  set Error.prototype.stack

  [...]
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).

  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  2. If SameValue(this, home) is true, then
    a. Throw a TypeError exception.
  3. Let desc be ? this.[[GetOwnProperty]](p).
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).
  5. Else,
    a. Perform ? Set(this, p, v, true).
includes: [proxyTrapsHelper.js]
features: [error-stack-accessor, Proxy]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

var trapLog = [];
var p = new Proxy(Error.prototype, allowProxyTraps({
  getOwnPropertyDescriptor: function (t, key) {
    trapLog.push(['gOPD', key]);
    return Object.getOwnPropertyDescriptor(t, key);
  },
  set: function (t, key, value) {
    trapLog.push(['set', key, value]);
    // Don't actually mutate Error.prototype; just acknowledge.
    return true;
  }
}));

// SameValue(p, Error.prototype) is false: a Proxy is a distinct object from
// its target. Step 2 does not throw; the algorithm proceeds to query the
// proxy's traps.
set.call(p, 'sentinel');

assert(trapLog.length >= 1, 'at least one trap was invoked');
assert.sameValue(trapLog[0][0], 'gOPD', 'getOwnPropertyDescriptor trap fired first');
assert.sameValue(trapLog[0][1], 'stack', 'getOwnPropertyDescriptor trap was called for "stack"');

// Error.prototype's own "stack" descriptor is an accessor, so step 5 of
// SetterThatIgnoresPrototypeProperties takes the Set path, which invokes the
// proxy's set trap.
var sawSet = false;
for (var i = 0; i < trapLog.length; ++i) {
  if (trapLog[i][0] === 'set') {
    sawSet = true;
    assert.sameValue(trapLog[i][1], 'stack', 'set trap was called for "stack"');
    assert.sameValue(trapLog[i][2], 'sentinel', 'set trap received the value');
  }
}
assert.sameValue(sawSet, true, 'set trap was invoked');
