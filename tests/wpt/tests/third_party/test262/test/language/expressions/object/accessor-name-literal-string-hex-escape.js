// This file was procedurally generated from the following sources:
// - src/accessor-names/literal-string-hex-escape.case
// - src/accessor-names/default/obj.template
/*---
description: Computed values as accessor property names (string literal containing a hexadecimal escape sequence) (Object initializer)
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
  get ['hex\x45scape']() { return 'get string'; },
  set ['hex\x45scape'](param) { stringSet = param; }
};

assert.sameValue(obj['hexEscape'], 'get string');

obj['hexEscape'] = 'set string';
assert.sameValue(stringSet, 'set string');
