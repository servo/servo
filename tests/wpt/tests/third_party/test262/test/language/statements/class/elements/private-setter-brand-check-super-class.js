// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Subclass can access private methods of a superclass (private setter)
esid: sec-privatefieldget
info: |
  SuperCall : super Arguments
    ...
    10. Perform ? InitializeInstanceElements(result, F).
    ...

  InitializeInstanceFieldsElements ( O, constructor )
    1. Assert: Type ( O ) is Object.
    2. Assert: Assert constructor is an ECMAScript function object.
    3. If constructor.[[PrivateBrand]] is not undefined,
      a. Perform ? PrivateBrandAdd(O, constructor.[[PrivateBrand]]).
    4. Let fieldRecords be the value of constructor's [[Fields]] internal slot.
    5. For each item fieldRecord in order from fieldRecords,
      a. Perform ? DefineField(O, fieldRecord).
    6. Return.
features: [class, class-methods-private]
---*/

class S {
  set #m(v) { this._v = v }
  
  superAccess(v) { this.#m = v; }
}

class C extends S {
  set #m(v) { this._u = v; }
  
  access(v) {
    return this.#m = v;
  }
}
  
let c = new C();

c.access('test262');
assert.sameValue(c._u, 'test262');

c.superAccess('super class');
assert.sameValue(c._v, 'super class');

let s = new S();
s.superAccess('super class')
assert.sameValue(s._v, 'super class');

assert.throws(TypeError, function() {
  c.access.call(s, 'foo');
}, 'invalid access of C private method');
