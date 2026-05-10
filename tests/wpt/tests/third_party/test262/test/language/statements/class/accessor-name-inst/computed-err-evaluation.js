// This file was procedurally generated from the following sources:
// - src/accessor-names/computed-err-evaluation.case
// - src/accessor-names/error/cls-decl-inst.template
/*---
description: Abrupt completion when evaluating expression (Class declaration, instance method)
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

    ComputedPropertyName : [ AssignmentExpression ]

    1. Let exprValue be the result of evaluating AssignmentExpression.
    2. Let propName be ? GetValue(exprValue).
---*/
var thrower = function() {
  throw new Test262Error();
};


assert.throws(Test262Error, function() {
  class C {
    get [thrower()]() {}
  }
}, '`get` accessor');

assert.throws(Test262Error, function() {
  class C {
    set [thrower()](_) {}
  }
}, '`set` accessor');
