// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Class construction should error if evaluation of field initializer in super errors
esid: sec-super-keyword-runtime-semantics-evaluation
info: |
  Runtime Semantics: Evaluation
    SuperCall : superArguments
      1. Let newTarget be GetNewTarget().
      2. If newTarget is undefined, throw a ReferenceError exception.
      3. Let func be ? GetSuperConstructor().
      4. Let argList be ArgumentListEvaluation of Arguments.
      5. ReturnIfAbrupt(argList).
      6. Let result be ? Construct(func, argList, newTarget).
      7. Let thisER be GetThisEnvironment( ).
      8. Let F be thisER.[[FunctionObject]].
      9. Assert: F is an ECMAScript function object.
      10. Perform ? InitializeInstanceFields(result, F).

  InitializeInstanceFields ( O, constructor )
    1. Assert: Type ( O ) is Object.
    2. Assert: Assert constructor is an ECMAScript function object.
    3. Let fieldRecords be the value of constructor's [[Fields]] internal slot.
    4. For each item fieldRecord in order from fieldRecords,
      a. If fieldRecord.[[static]] is false, then
        i. Perform ? DefineField(O, fieldRecord).

  DefineField(receiver, fieldRecord)
    1. Assert: Type(receiver) is Object.
    2. Assert: fieldRecord is a Record as created by ClassFieldDefinitionEvaluation.
    3. Let fieldName be fieldRecord.[[Name]].
    4. Let initializer be fieldRecord.[[Initializer]].
    5. If initializer is not empty, then
        a.Let initValue be ? Call(initializer, receiver).

features: [class, class-fields-public]
---*/

function f() {
  throw new Test262Error();
}

class A {
  x = f();
}

class C extends A {
  constructor() {
    super();
  }
}

assert.throws(Test262Error, function() {
  new C();
})
