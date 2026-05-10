// This file was procedurally generated from the following sources:
// - src/accessor-names/computed-err-unresolvable.case
// - src/accessor-names/error/cls-expr-static.template
/*---
description: Abrupt completion when resolving reference value (Class expression, static method)
esid: sec-runtime-semantics-classdefinitionevaluation
features: [class]
flags: [generated]
info: |
    [...]
    21. For each ClassElement m in order from methods
        a. If IsStatic of m is false, then
           [...]
        b. Else,
           a. Let status be the result of performing PropertyDefinitionEvaluation
              for m with arguments F and false.


    12.2.6.7 Runtime Semantics: Evaluation

    [...]

    ComputedPropertyName : [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
---*/

assert.throws(ReferenceError, function() {
  0, class {
    static get [test262unresolvable]() {}
  };
}, '`get` accessor');

assert.throws(ReferenceError, function() {
  0, class {
    static set [test262unresolvable](_) {}
  };
}, '`set` accessor');
