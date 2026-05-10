// This file was procedurally generated from the following sources:
// - src/accessor-names/computed-err-evaluation.case
// - src/accessor-names/error/obj.template
/*---
description: Abrupt completion when evaluating expression (Object initializer)
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

    ComputedPropertyName : [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
---*/
var thrower = function() {
  throw new Test262Error();
};


assert.throws(Test262Error, function() {
  ({
    get [thrower()]() {}
  });
}, '`get` accessor');

assert.throws(Test262Error, function() {
  ({
    set [thrower()](_) {}
  });
}, '`set` accessor');
