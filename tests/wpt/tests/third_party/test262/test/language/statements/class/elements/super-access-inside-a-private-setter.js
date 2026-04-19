// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Private setter contains proper HomeObject
esid: sec-method-definitions-runtime-semantics-classelementevaluation
info: |
  MethodDefinition : set ClassElementName ( PropertySetParameterList ) { FunctionBody }
    1. Let key be the result of evaluating ClassElementName.
    2. ReturnIfAbrupt(key).
    3. If the function code for this MethodDefinition is strict mode code, let strict be true. Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be FunctionCreate(Method, PropertySetParameterList, FunctionBody, scope, strict).
    6. Perform MakeMethod(closure, homeObject).
    7. Perform SetFunctionName(closure, key, "set").
    8. If key is a Private Name,
      a. If key has a [[Kind]] field,
        i. Assert: key.[[Kind]] is "accessor".
        ii. Assert: key.[[Brand]] is homeObject.
        iii. Assert: key does not have a [[Set]] field.
        iv. Set key.[[Set]] to closure.
      b. Otherwise,
        i. Set key.[[Kind]] to "accessor".
        ii. Set key.[[Brand]] to homeObject.
        iii. Set key.[[Set]] to closure.
    9. Else,
      a. Let desc be the PropertyDescriptor{[[Set]]: closure, [[Enumerable]]: enumerable, [[Configurable]]: true}.
      b. Perform ? DefinePropertyOrThrow(homeObject, key, desc).
features: [class-methods-private, class]
---*/

class A {
  method(v) {
    return v;
  }
}

class C extends A {
  set #m(v) {
    this._v = super.method(v);
  }

  access() {
    return this.#m = "Test262";
  }
}

let c = new C();
c.access();
assert.sameValue(c._v, "Test262");
