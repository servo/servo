// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: SuperProperty's behavior as a strict reference
info: |
  1. Let propertyKey be StringValue of IdentifierName.
  2. If the code matched by the syntactic production that is being evaluated is
     strict mode code, let strict be true, else let strict be false.
  3. Return ? MakeSuperPropertyReference(propertyKey, strict).

  12.3.5.3 Runtime Semantics: MakeSuperPropertyReference

  1. Let env be GetThisEnvironment( ).
  2. If env.HasSuperBinding() is false, throw a ReferenceError exception.
  3. Let actualThis be ? env.GetThisBinding().
  4. Let baseValue be ? env.GetSuperBase().
  5. Let bv be ? RequireObjectCoercible(baseValue).
  6. Return a value of type Reference that is a Super Reference whose base
     value component is bv, whose referenced name component is propertyKey,
     whose thisValue component is actualThis, and whose strict reference flag
     is strict.

  6.2.3.2 PutValue

  [...]
  5. If IsUnresolvableReference(V) is true, then
     [...]
  6. Else if IsPropertyReference(V) is true, then
     a. If HasPrimitiveBase(V) is true, then
        [...]
     b. Let succeeded be ? base.[[Set]](GetReferencedName(V), W,
        GetThisValue(V)).
     c. If succeeded is false and IsStrictReference(V) is true, throw a
        TypeError exception.
features: [class]
---*/

var caught;
class C {
  method() {
    super.x = 8;
    Object.freeze(C.prototype);
    try {
      super.y = 9;
    } catch (err) {
      caught = err;
    }
  }
}

C.prototype.method();

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, TypeError);
