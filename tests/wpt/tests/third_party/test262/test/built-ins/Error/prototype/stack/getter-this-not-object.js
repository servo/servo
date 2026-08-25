// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-error.prototype.stack
description: >
  Throws a TypeError if the this value is not an Object.
info: |
  get Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
features: [error-stack-accessor]
---*/

var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;

assert.sameValue(typeof get, 'function');

var badReceivers = [
  ['undefined', undefined],
  ['null', null],
  ['true', true],
  ['false', false],
  ['number', 1],
  ['string', ''],
  typeof Symbol === 'undefined' ? null : ['symbol', Symbol('s')],
  typeof BigInt === 'undefined' ? null : ['bigint', BigInt(0)]
];

for (var i = 0; i < badReceivers.length; ++i) {
  if (!badReceivers[i]) continue;
  var label = badReceivers[i][0];
  var value = badReceivers[i][1];
  assert.throws(TypeError, function () {
    get.call(value);
  }, label);
}
