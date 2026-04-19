// This file was procedurally generated from the following sources:
// - src/accessor-names/literal-string-double-quote.case
// - src/accessor-names/default/cls-decl-inst.template
/*---
description: Computed values as accessor property names (string literal using double quotes) (Class declaration, instance method)
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
---*/

var stringSet;

class C {
  get "doubleQuote"() { return 'get string'; }
  set "doubleQuote"(param) { stringSet = param; }
}

assert.sameValue(C.prototype["doubleQuote"], 'get string');

C.prototype["doubleQuote"] = 'set string';
assert.sameValue(stringSet, 'set string');
