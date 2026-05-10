// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Subclass can access private methods of a superclass (private getter)
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
  get #m() { return 'super class'; }
  
  superAccess() { return this.#m; }
}

class C extends S {
  get #m() { return 'test262'; }
  
  access() {
    return this.#m;
  }
}
  
let c = new C();

assert.sameValue(c.access(), 'test262');
assert.sameValue(c.superAccess(), 'super class');

let s = new S();
assert.sameValue(s.superAccess(), 'super class');
assert.throws(TypeError, function() {
  c.access.call(s);
}, 'invalid access of C private method');
