// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: SuperProperty's "this" value
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

  GetValue (V)

  1. ReturnIfAbrupt(V).
  2. If Type(V) is not Reference, return V.
  3. Let base be GetBase(V).
  4. If IsUnresolvableReference(V) is true, throw a ReferenceError exception.
  5. If IsPropertyReference(V) is true, then
     a. If HasPrimitiveBase(V) is true, then
        i. Assert: In this case, base will never be null or undefined.
        ii. Let base be ! ToObject(base).
     b. Return ? base.[[Get]](GetReferencedName(V), GetThisValue(V)).
---*/

var viaCall;
var viaMember;
class Parent {
  getThis() {
    return this;
  }
  get This() {
    return this;
  }
}
class C extends Parent {
  method() {
    viaCall = super.getThis();
    viaMember = super.This;
  }
}

C.prototype.method();

assert.sameValue(viaCall, C.prototype, 'via CallExpression');
assert.sameValue(viaMember, C.prototype, 'via MemberExpression');
