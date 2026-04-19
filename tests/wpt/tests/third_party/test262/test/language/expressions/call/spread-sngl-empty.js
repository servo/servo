// This file was procedurally generated from the following sources:
// - src/spread/sngl-empty.case
// - src/spread/default/call-expr.template
/*---
description: Spread operator applied to the only argument when no iteration occurs (CallExpression)
esid: sec-function-calls-runtime-semantics-evaluation
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
    6. Repeat
       a. Let next be IteratorStep(iterator).
       b. ReturnIfAbrupt(next).
       c. If next is false, return list.
       [...]
---*/

var callCount = 0;

(function() {
  assert.sameValue(arguments.length, 0);
  callCount += 1;
}(...[]));

assert.sameValue(callCount, 1);
