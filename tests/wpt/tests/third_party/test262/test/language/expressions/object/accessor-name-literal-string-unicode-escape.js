// This file was procedurally generated from the following sources:
// - src/accessor-names/literal-string-unicode-escape.case
// - src/accessor-names/default/obj.template
/*---
description: Computed values as accessor property names (string literal containing a Unicode escape sequence) (Object initializer)
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
  get ['unicod\u{000065}Escape']() { return 'get string'; },
  set ['unicod\u{000065}Escape'](param) { stringSet = param; }
};

assert.sameValue(obj['unicodeEscape'], 'get string');

obj['unicodeEscape'] = 'set string';
assert.sameValue(stringSet, 'set string');
