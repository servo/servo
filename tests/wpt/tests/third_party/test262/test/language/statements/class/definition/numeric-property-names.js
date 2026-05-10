// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class numeric property names
---*/
function assertMethodDescriptor(object, name) {
  var desc = Object.getOwnPropertyDescriptor(object, name);
  assert.sameValue(desc.configurable, true, "The value of `desc.configurable` is `true`");
  assert.sameValue(desc.enumerable, false, "The value of `desc.enumerable` is `false`");
  assert.sameValue(desc.writable, true, "The value of `desc.writable` is `true`");
  assert.sameValue(typeof desc.value, 'function', "`typeof desc.value` is `'function'`");
  assert.sameValue('prototype' in desc.value, false, "The result of `'prototype' in desc.value` is `false`");
}

function assertGetterDescriptor(object, name) {
  var desc = Object.getOwnPropertyDescriptor(object, name);
  assert.sameValue(desc.configurable, true, "The value of `desc.configurable` is `true`");
  assert.sameValue(desc.enumerable, false, "The value of `desc.enumerable` is `false`");
  assert.sameValue(typeof desc.get, 'function', "`typeof desc.get` is `'function'`");
  assert.sameValue('prototype' in desc.get, false, "The result of `'prototype' in desc.get` is `false`");
  assert.sameValue(desc.set, undefined, "The value of `desc.set` is `undefined`");
}

function assertSetterDescriptor(object, name) {
  var desc = Object.getOwnPropertyDescriptor(object, name);
  assert.sameValue(desc.configurable, true, "The value of `desc.configurable` is `true`");
  assert.sameValue(desc.enumerable, false, "The value of `desc.enumerable` is `false`");
  assert.sameValue(typeof desc.set, 'function', "`typeof desc.set` is `'function'`");
  assert.sameValue('prototype' in desc.set, false, "The result of `'prototype' in desc.set` is `false`");
  assert.sameValue(desc.get, undefined, "The value of `desc.get` is `undefined`");
}

class B {
  1() { return 1; }
  get 2() { return 2; }
  set 3(_) {}

  static 4() { return 4; }
  static get 5() { return 5; }
  static set 6(_) {}
}

assertMethodDescriptor(B.prototype, '1');
assertGetterDescriptor(B.prototype, '2');
assertSetterDescriptor(B.prototype, '3');

assertMethodDescriptor(B, '4');
assertGetterDescriptor(B, '5');
assertSetterDescriptor(B, '6');

class C extends B {
  1() { return super[1](); }
  get 2() { return super[2]; }
  static 4() { return super[4](); }
  static get 5() { return super[5]; }
}

assert.sameValue(new C()[1](), 1, "`new C()[1]()` returns `1`. Defined as `1() { return super[1](); }`");
assert.sameValue(new C()[2], 2, "The value of `new C()[2]` is `2`. Defined as `get 2() { return super[2]; }`");
assert.sameValue(C[4](), 4, "`C[4]()` returns `4`. Defined as `static 4() { return super[4](); }`");
assert.sameValue(C[5], 5, "The value of `C[5]` is `5`. Defined as `static get 5() { return super[5]; }`");
