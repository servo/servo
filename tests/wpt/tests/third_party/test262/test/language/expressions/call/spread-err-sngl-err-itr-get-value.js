// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-itr-get-value.case
// - src/spread/error/call-expr.template
/*---
description: Spread operator applied to the only argument when GetIterator fails (@@iterator function return value) (CallExpression)
esid: sec-function-calls-runtime-semantics-evaluation
features: [Symbol.iterator]
flags: [generated]
info: |
    CallExpression : MemberExpression Arguments

    [...]
    9. Return EvaluateDirectCall(func, thisValue, Arguments, tailCall).

    12.3.4.3 Runtime Semantics: EvaluateDirectCall

    1. Let argList be ArgumentListEvaluation(arguments).
    [...]
    6. Let result be Call(func, thisValue, argList).
    [...]

    12.3.6.1 Runtime Semantics: ArgumentListEvaluation

    ArgumentList : ... AssignmentExpression

    1. Let list be an empty List.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let spreadObj be GetValue(spreadRef).
    4. Let iterator be GetIterator(spreadObj).
    5. ReturnIfAbrupt(iterator).

    7.4.1 GetIterator ( obj, method )

    [...]
    2. Let iterator be ? Call(method, obj).
    3. If Type(iterator) is not Object, throw a TypeError exception.
---*/
var iter = {};
iter[Symbol.iterator] = function() {
  return null;
};

assert.throws(TypeError, function() {
  (function() {}(...iter));
});
