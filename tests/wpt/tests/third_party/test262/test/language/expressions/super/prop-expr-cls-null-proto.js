// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: >
  SuperProperty evaluation when the "home" object's prototype is not
  object-coercible.
info: |
  [...]
  4. If the code matched by the syntactic production that is being evaluated is
     strict mode code, let strict be true, else let strict be false.
  5. Return ? MakeSuperPropertyReference(propertyKey, strict).

  12.3.5.3 Runtime Semantics: MakeSuperPropertyReference

  1. Let env be GetThisEnvironment( ).
  2. If env.HasSuperBinding() is false, throw a ReferenceError exception.
  3. Let actualThis be ? env.GetThisBinding().
  4. Let baseValue be ? env.GetSuperBase().
  5. Let bv be ? RequireObjectCoercible(baseValue).
features: [class]
---*/

var caught;
class C extends null {
  method() {
    try {
      super['x'];
    } catch (err) {
      caught = err;
    }
  }
}

C.prototype.method();

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, TypeError);
