// This file was procedurally generated from the following sources:
// - src/spread/mult-err-iter-get-value.case
// - src/spread/error/call-expr.template
/*---
description: Spread operator following other arguments when GetIterator fails (@@iterator function return value) (CallExpression)
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

    ArgumentList : ArgumentList , ... AssignmentExpression

    1. Let precedingArgs be the result of evaluating ArgumentList.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let iterator be GetIterator(GetValue(spreadRef) ).
    4. ReturnIfAbrupt(iterator).

    7.4.1 GetIterator ( obj, method )

    [...]
    2. Let iterator be ? Call(method, obj).
    3. If Type(iterator) is not Object, throw a TypeError exception.
---*/
var iter = {};
Object.defineProperty(iter, Symbol.iterator, {
  get: function() {
    return null;
  }
});

assert.throws(TypeError, function() {
  (function() {}(0, ...iter));
});
