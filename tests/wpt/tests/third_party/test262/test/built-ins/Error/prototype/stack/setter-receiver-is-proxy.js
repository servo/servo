// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  The setter calls [[GetOwnProperty]] and either [[DefineOwnProperty]] (via
  CreateDataPropertyOrThrow) or [[Set]] on the receiver, observably invoking
  Proxy traps.
info: |
  SetterThatIgnoresPrototypeProperties ( this, home, p, v )

  [...]
  3. Let desc be ? this.[[GetOwnProperty]](p).
  4. If desc is undefined, then
    a. Perform ? CreateDataPropertyOrThrow(this, p, v).
  5. Else,
    a. Perform ? Set(this, p, v, true).
includes: [proxyTrapsHelper.js]
features: [error-stack-accessor, Proxy, Reflect]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

// (a) Proxy with no own "stack": getOwnPropertyDescriptor then defineProperty.
// allowProxyTraps throws Test262Error for any other trap (including [[Set]]).
var trapLog = [];
var target1 = {};
var p1 = new Proxy(target1, allowProxyTraps({
  getOwnPropertyDescriptor: function (t, key) {
    trapLog.push(['gOPD', key]);
    return Object.getOwnPropertyDescriptor(t, key);
  },
  defineProperty: function (t, key, desc) {
    trapLog.push(['define', key, desc]);
    return Reflect.defineProperty(t, key, desc);
  }
}, '(a)'));

set.call(p1, 'sentinel');
assert(trapLog.length >= 2, 'at least getOwnPropertyDescriptor and defineProperty were called');
assert.sameValue(trapLog[0][0], 'gOPD', 'first trap is getOwnPropertyDescriptor');
assert.sameValue(trapLog[0][1], 'stack', 'getOwnPropertyDescriptor was called for "stack"');

var lastDefine = null;
for (var i = 0; i < trapLog.length; ++i) {
  if (trapLog[i][0] === 'define') {
    lastDefine = trapLog[i];
  }
}
assert.notSameValue(lastDefine, null, 'defineProperty trap was invoked');
assert.sameValue(lastDefine[1], 'stack', 'defineProperty was called for "stack"');

// CreateDataProperty constructs the descriptor record
// { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true, [[Configurable]]: true };
// FromPropertyDescriptor converts that into a plain Object with exactly those
// four own properties before invoking the defineProperty trap.
var passedDesc = lastDefine[2];
assert.sameValue(passedDesc.value, 'sentinel', 'descriptor.value');
assert.sameValue(passedDesc.writable, true, 'descriptor.writable');
assert.sameValue(passedDesc.enumerable, true, 'descriptor.enumerable');
assert.sameValue(passedDesc.configurable, true, 'descriptor.configurable');
assert.sameValue('get' in passedDesc, false, 'descriptor has no get key');
assert.sameValue('set' in passedDesc, false, 'descriptor has no set key');
assert.sameValue(target1.stack, 'sentinel', 'value reached the underlying target');

// (b) Proxy with own "stack": getOwnPropertyDescriptor then set trap.
// allowProxyTraps throws Test262Error for any other trap (including
// [[DefineOwnProperty]]).
var trapLog2 = [];
var target2 = { stack: 'old' };
var p2 = new Proxy(target2, allowProxyTraps({
  getOwnPropertyDescriptor: function (t, key) {
    trapLog2.push(['gOPD', key]);
    return Object.getOwnPropertyDescriptor(t, key);
  },
  set: function (t, key, value) {
    trapLog2.push(['set', key, value]);
    t[key] = value;
    return true;
  }
}, '(b)'));

set.call(p2, 'updated');
var sawSet = false;
for (var j = 0; j < trapLog2.length; ++j) {
  if (trapLog2[j][0] === 'set') {
    sawSet = true;
    assert.sameValue(trapLog2[j][1], 'stack', 'set was called for "stack"');
    assert.sameValue(trapLog2[j][2], 'updated', 'set received the value');
  }
}
assert.sameValue(sawSet, true, 'set trap was invoked');
assert.sameValue(target2.stack, 'updated', 'value reached the underlying target');
