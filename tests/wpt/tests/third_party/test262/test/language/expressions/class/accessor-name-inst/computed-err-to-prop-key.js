// This file was procedurally generated from the following sources:
// - src/accessor-names/computed-err-to-prop-key.case
// - src/accessor-names/error/cls-expr-inst.template
/*---
description: Abrupt completion when coercing to property key value (Class expression, instance method)
esid: sec-runtime-semantics-classdefinitionevaluation
features: [class]
flags: [generated]
info: |
    [...]
    21. For each ClassElement m in order from methods
        a. If IsStatic of m is false, then
           i. Let status be the result of performing PropertyDefinitionEvaluation
              for m with arguments proto and false.


    12.2.6.7 Runtime Semantics: Evaluation

    [...]

    ComputedPropertyName : [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
    3. Return ? ToPropertyKey(propName).

    7.1.14 ToPropertyKey

    1. Let key be ? ToPrimitive(argument, hint String).

    7.1.1 ToPrimitive

    [...]
    7. Return ? OrdinaryToPrimitive(input, hint).

    7.1.1.1 OrdinaryToPrimitive

    5. For each name in methodNames in List order, do
       [...]
    6. Throw a TypeError exception.
---*/
var badKey = Object.create(null);


assert.throws(TypeError, function() {
  0, class {
    get [badKey]() {}
  };
}, '`get` accessor');

assert.throws(TypeError, function() {
  0, class {
    set [badKey](_) {}
  };
}, '`set` accessor');
