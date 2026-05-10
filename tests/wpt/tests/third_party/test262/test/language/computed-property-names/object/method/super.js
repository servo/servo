// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property methods can call super methods
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
  ['a']() { return 'a' + super.m(); },
  [ID('b')]() { return 'b' + super.m(); },
  [0]() { return '0' + super.m(); },
  [ID(1)]() { return '1' + super.m(); },
};

Object.setPrototypeOf(object, proto);

assert.sameValue(object.a(), 'a proto m', "`object.a()` returns `'a proto m'`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(object.b(), 'b proto m', "`object.b()` returns `'b proto m'`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(object[0](), '0 proto m', "`object[0]()` returns `'0 proto m'`, after executing `Object.setPrototypeOf(object, proto);`");
assert.sameValue(object[1](), '1 proto m', "`object[1]()` returns `'1 proto m'`, after executing `Object.setPrototypeOf(object, proto);`");
