// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-propertydefinitionevaluation
description: >
  ToPropertyKey is performed before evaluating the value expression.
info: |
  13.2.5.5 Runtime Semantics: PropertyDefinitionEvaluation

    PropertyDefinition : PropertyName : AssignmentExpression

    1. Let propKey be ? Evaluation of PropertyName.
    ...
    6. Else,
      a. Let exprValueRef be ? Evaluation of AssignmentExpression.
      b. Let propValue be ? GetValue(exprValueRef).
    ...
    9. Perform ! CreateDataPropertyOrThrow(object, propKey, propValue).
    ...

  13.2.5.4 Runtime Semantics: Evaluation

    ComputedPropertyName : [ AssignmentExpression ]

    1. Let exprValue be ? Evaluation of AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
    3. Return ? ToPropertyKey(propName).
---*/

var value = "bad";

var key = {
  toString() {
    value = "ok";
    return "p";
  }
};

var obj = {
  [key]: value
};

assert.sameValue(obj.p, "ok");
