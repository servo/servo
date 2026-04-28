// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Private method contains proper HomeObject
esid: sec-method-definitions-runtime-semantics-classelementevaluation
info: |
  MethodDefinition : ClassElementName ( UniqueFormalParameters ) { FunctionBody }
    1. Let methodDef be DefineMethod of MethodDefinition with argument homeObject.
    2. ReturnIfAbrupt(methodDef).
    3. Perform ? DefineOrdinaryMethod(methodDef.[[Key]], homeObject, methodDef.[[Closure]], _enumerable).

  MethodDefinition : PropertyName ( UniqueFormalParameters ) { FunctionBody }
    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. Let scope be the running execution context's LexicalEnvironment.
    4. If functionPrototype is present as a parameter, then
      a. Let kind be Normal.
      b. Let prototype be functionPrototype.
    5. Else,
      a. Let kind be Method.
      b. Let prototype be the intrinsic object %FunctionPrototype%.
    6. Let closure be FunctionCreate(kind, UniqueFormalParameters, FunctionBody, scope, prototype).
    7. Perform MakeMethod(closure, object).
    8. Set closure.[[SourceText]] to the source text matched by MethodDefinition.
    9. Return the Record { [[Key]]: propKey, [[Closure]]: closure }.
features: [class-methods-private, class]
---*/

class A {
  method() {
    return "Test262";
  }
}

class C extends A {
  #m() {
    return super.method();
  }

  access(o) {
    return this.#m.call(o);
  }
}

let c = new C();
assert.sameValue(c.access(c), "Test262");

let o = {};
assert.sameValue(c.access(o), "Test262");
