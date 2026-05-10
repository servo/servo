// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Private getter contains proper HomeObject
esid: sec-method-definitions-runtime-semantics-classelementevaluation
info: |
  MethodDefinition : get ClassElementName () { FunctionBody }
    1. Let key be the result of evaluating ClassElementName.
    2. ReturnIfAbrupt(key).
    3. If the function code for this MethodDefinition is strict mode code, let strict be true. Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let formalParameterList be an instance of the production FormalParameters:[empty] .
    6. Let closure be FunctionCreate(Method, formalParameterList, FunctionBody, scope, strict).
    7. Perform MakeMethod(closure, homeObject).
    8. Perform SetFunctionName(closure, key, "get").
    9. If key is a Private Name,
      a. If key has a [[Kind]] field,
        i. Assert: key.[[Kind]] is "accessor".
        ii. Assert: key.[[Brand]] is homeObject.
        iii. Assert: key does not have a [[Get]] field.
        iv. Set key.[[Get]] to closure.
      b. Otherwise,
        i. Set key.[[Kind]] to "accessor".
        ii. Set key.[[Brand]] to homeObject.
        iii. Set key.[[Get]] to closure.
    10. Else,
      a. Let desc be the PropertyDescriptor{[[Get]]: closure, [[Enumerable]]: enumerable, [[Configurable]]: true}.
      b. Perform ? DefinePropertyOrThrow(homeObject, key, desc).
features: [class-methods-private, class]
---*/

class A {
  method() {
    return "Test262";
  }
}

class C extends A {
  get #m() {
    return super.method();
  }

  access() {
    return this.#m;
  }
}

let c = new C();
assert.sameValue(c.access(), "Test262");
