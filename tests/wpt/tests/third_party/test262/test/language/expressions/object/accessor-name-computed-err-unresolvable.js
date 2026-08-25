// This file was procedurally generated from the following sources:
// - src/accessor-names/computed-err-unresolvable.case
// - src/accessor-names/error/obj.template
/*---
description: Abrupt completion when resolving reference value (Object initializer)
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
---*/

assert.throws(ReferenceError, function() {
  ({
    get [test262unresolvable]() {}
  });
}, '`get` accessor');

assert.throws(ReferenceError, function() {
  ({
    set [test262unresolvable](_) {}
  });
}, '`set` accessor');
