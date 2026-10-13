// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  Throws a TypeError if the this value is not an Object.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

assert.sameValue(typeof set, 'function');

var badReceivers = [
  ['undefined', undefined],
  ['null', null],
  ['true', true],
  ['false', false],
  ['number', 1],
  ['string', 's'],
  typeof Symbol === 'undefined' ? null : ['symbol', Symbol('s')],
  typeof BigInt === 'undefined' ? null : ['bigint', BigInt(0)]
];

for (var i = 0; i < badReceivers.length; ++i) {
  if (!badReceivers[i]) continue;
  var label = badReceivers[i][0];
  var value = badReceivers[i][1];
  assert.throws(TypeError, function () {
    set.call(value, '');
  }, label);
}

// A non-Object this combined with a non-String v throws TypeError. The spec
// runs step 2 (this not Object) before step 3 (v not String), but both steps
// throw TypeError, so the failure is observably the same either way.
assert.throws(TypeError, function () {
  set.call(undefined, 0);
}, 'undefined this with non-String v');

assert.throws(TypeError, function () {
  set.call(null, {});
}, 'null this with non-String v');
