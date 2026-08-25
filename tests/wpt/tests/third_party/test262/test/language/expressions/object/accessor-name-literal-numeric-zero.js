// This file was procedurally generated from the following sources:
// - src/accessor-names/literal-numeric-zero.case
// - src/accessor-names/default/obj.template
/*---
description: Computed values as accessor property names (numeric literal zero) (Object initializer)
esid: sec-object-initializer-runtime-semantics-evaluation
flags: [generated]
info: |
    ObjectLiteral :
      { PropertyDefinitionList }
      { PropertyDefinitionList , }

    1. Let obj be ObjectCreate(%ObjectPrototype%).
    2. Let status be the result of performing PropertyDefinitionEvaluation of
       PropertyDefinitionList with arguments obj and true.

    12.2.6.7 Runtime Semantics: Evaluation

    [...]

    ComputedPropertyName : [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
    3. Return ? ToPropertyKey(propName).
---*/

var stringSet;
var obj = {
  get [0]() { return 'get string'; },
  set [0](param) { stringSet = param; }
};

assert.sameValue(obj['0'], 'get string');

obj['0'] = 'set string';
assert.sameValue(stringSet, 'set string');
