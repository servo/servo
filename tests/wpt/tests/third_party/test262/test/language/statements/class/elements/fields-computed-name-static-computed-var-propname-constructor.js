// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: static class fields forbid PropName 'constructor' (no early error -- PropName of ComputedPropertyName not forbidden value)
esid: sec-class-definitions-static-semantics-early-errors
features: [class, class-static-fields-public]
info: |
    Static Semantics: PropName
    ...
    ComputedPropertyName : [ AssignmentExpression ]
      Return empty.

    This test file tests the following early error is only valid for a matching PropName:

    Static Semantics: Early Errors

    ClassElement : static FieldDefinition;
        It is a Syntax Error if PropName of FieldDefinition is "prototype" or "constructor".

    -- IDK what is calling InitializeClassElements but I guess it's supposed to be called to
    -- set the fields

    InitializeClassElements(F, proto)

    ...
    6. For each item element in order from elements,
      a. If element.[[Kind]] is "field" and element.[[Placement]] is "static" or "prototype",
        ...
        ii. Let receiver be F if element.[[Placement]] is "static", else let receiver be proto.
        iii. Perform ? DefineClassElement(receiver, element).

    -- DefineClassElement is probably DefineField in the class fields proposal

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
includes: [propertyHelper.js]
---*/

var x = 'constructor';
class C1 {
  static [x];
}

verifyProperty(C1, 'constructor', {
  value: undefined,
  configurable: true,
  writable: true,
  enumerable: true,
});

class C2 {
  static [x] = 42;
}

verifyProperty(C2, 'constructor', {
  value: 42,
  configurable: true,
  writable: true,
  enumerable: true,
});
