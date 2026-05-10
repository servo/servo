// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property getters can call super methods
---*/

function ID(x) {
  return x;
}

var proto = {
  m() {
    return ' proto m';
  }
};
var object = {
  get ['a']() { return 'a' + super.m(); },
  get [ID('b')]() { return 'b' + super.m(); },
  get [0]() { return '0' + super.m(); },
  get [ID(1)]() { return '1' + super.m(); },
};

Object.setPrototypeOf(object, proto);

assert.sameValue(
  object.a,
  'a proto m',
  "The value of `object.a` is `'a proto m'`. Defined as `get ['a']() { return 'a' + super.m(); }`"
);
assert.sameValue(
  object.b,
  'b proto m',
  "The value of `object.b` is `'b proto m'`. Defined as `get [ID('b')]() { return 'b' + super.m(); }`"
);
assert.sameValue(
  object[0],
  '0 proto m',
  "The value of `object[0]` is `'0 proto m'`. Defined as `get [0]() { return '0' + super.m(); }`"
);
assert.sameValue(
  object[1],
  '1 proto m',
  "The value of `object[1]` is `'1 proto m'`. Defined as `get [ID(1)]() { return '1' + super.m(); }`"
);
