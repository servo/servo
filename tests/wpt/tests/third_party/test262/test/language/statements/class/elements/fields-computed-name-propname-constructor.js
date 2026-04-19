// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: class fields forbid PropName 'constructor' (no early error -- PropName of ComputedPropertyName not forbidden value)
esid: sec-class-definitions-static-semantics-early-errors
features: [class, class-fields-public]
info: |
    Static Semantics: PropName
    ...
    ComputedPropertyName : [ AssignmentExpression ]
      Return empty.


    // This test file tests the following early error:
    Static Semantics: Early Errors

      ClassElement : FieldDefinition;
        It is a Syntax Error if PropName of FieldDefinition is "constructor".

    DefineField(receiver, fieldRecord)

    ...
    8. If fieldName is a Private Name,
      ...
    9. Else,
      a. ...
      b. Perform ? CreateDataPropertyOrThrow(receiver, fieldName, initValue).

    CreateDataPropertyOrThrow ( O, P, V )

    ...
    3. Let success be ? CreateDataProperty(O, P, V).
    4. If success is false, throw a TypeError exception.
    ...

    CreateDataProperty ( O, P, V )

    ...
    3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true, [[Enumerable]]: true,
      [[Configurable]]: true }.
    4. Return ? O.[[DefineOwnProperty]](P, newDesc).
---*/

var x = "constructor";
class C1 {
  [x];
}

var c1 = new C1();

assert.sameValue(c1.hasOwnProperty("constructor"), true);
assert.sameValue(C1.hasOwnProperty("constructor"), false);
