// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Every new evaluation of a class creates a different Private Name (private static field)
esid: sec-runtime-semantics-evaluate-name
info: |
  ClassElementName : PrivateIdentifier
    1. Let privateIdentifier be StringValue of PrivateIdentifier.
    2. Let privateName be NewPrivateName(privateIdentifier).
    3. Let scope be the running execution context's PrivateEnvironment.
    4. Let scopeEnvRec be scope's EnvironmentRecord.
    5. Perform ! scopeEnvRec.InitializeBinding(privateIdentifier, privateName).
    6. Return privateName.

  ClassTail : ClassHeritage { ClassBody }
    ...
    27. Let staticFields be a new empty List.
    28. For each ClassElement e in order from elements,
      a. If IsStatic of e is false, then
        ...
      b. Else,
        i. Let field be the result of performing PropertyDefinitionEvaluation for m ClassElementEvaluation for e with arguments F and false.
      c. If field is an abrupt completion, then
        ...
      d. If field is not empty,
        i. If IsStatic of e is false, append field to instanceFields.
        ii. Otherwise, append field to staticFields.
    ...
    34. For each item fieldRecord in order from staticFields,
      a. Perform ? DefineField(F, field).
    ...

  DefineField(receiver, fieldRecord)
    ...
    8. If fieldName is a Private Name,
      a. Perform ? PrivateFieldAdd(fieldName, receiver, initValue).
features: [class, class-static-fields-private]
flags: [noStrict]
---*/

let classStringExpression = `(
class {
  static #m = 'test262';

  static access() {
    return this.#m;
  }
}
)`;

let evalClass = function (_eval) {
  return _eval(classStringExpression);
};

let C1 = evalClass(eval);
let C2 = evalClass(eval);

assert.sameValue(C1.access(), 'test262');
assert.sameValue(C2.access(), 'test262');

assert.throws(TypeError, function() {
  C1.access.call(C2);
}, 'invalid access of c1 private static field');

assert.throws(TypeError, function() {
  C2.access.call(C1);
}, 'invalid access of c2 private static field');
