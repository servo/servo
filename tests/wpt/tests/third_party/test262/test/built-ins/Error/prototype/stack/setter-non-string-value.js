// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  Throws a TypeError if the value being assigned is not a String.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
includes: [nativeErrors.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

// An object with a toString method is not coerced; the algorithm requires v
// to already be a String.
var coercible = {
  toString: function () { return 'coerced'; },
  valueOf: function () { return 'coerced'; },
};

var badValues = [
  ['undefined', undefined],
  ['null', null],
  ['true', true],
  ['false', false],
  ['number', 1],
  ['object', {}],
  ['array', []],
  ['object with toString', coercible],
  ['String wrapper object', new String('boxed')],
  typeof Symbol === 'undefined' ? null : ['symbol', Symbol('s')],
  typeof BigInt === 'undefined' ? null : ['bigint', BigInt(0)]
];

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');

  for (var j = 0; j < badValues.length; ++j) {
    if (!badValues[j]) continue;
    var label = badValues[j][0];
    var value = badValues[j][1];
    assert.throws(TypeError, function () {
      set.call(err, value);
    }, Ctor.name + ': ' + label);
  }
}
