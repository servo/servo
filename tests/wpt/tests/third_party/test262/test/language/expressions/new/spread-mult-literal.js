// This file was procedurally generated from the following sources:
// - src/spread/mult-literal.case
// - src/spread/default/member-expr.template
/*---
description: Spread operator applied to AssignmentExpression following other elements (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
flags: [generated]
info: |
    MemberExpression : new MemberExpression Arguments

    1. Return EvaluateNew(MemberExpression, Arguments).

    12.3.3.1.1 Runtime Semantics: EvaluateNew

    6. If arguments is empty, let argList be an empty List.
    7. Else,
       a. Let argList be ArgumentListEvaluation of arguments.
       [...]

    12.3.6.1 Runtime Semantics: ArgumentListEvaluation

    ArgumentList : ArgumentList , ... AssignmentExpression

    1. Let precedingArgs be the result of evaluating ArgumentList.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let iterator be GetIterator(GetValue(spreadRef) ).
    4. ReturnIfAbrupt(iterator).
    5. Repeat
       a. Let next be IteratorStep(iterator).
       b. ReturnIfAbrupt(next).
       c. If next is false, return precedingArgs.
---*/

var callCount = 0;

new function() {
  assert.sameValue(arguments.length, 5);
  assert.sameValue(arguments[0], 5);
  assert.sameValue(arguments[1], 6);
  assert.sameValue(arguments[2], 7);
  assert.sameValue(arguments[3], 8);
  assert.sameValue(arguments[4], 9);
  callCount += 1;
}(5, ...[6, 7, 8], 9);

assert.sameValue(callCount, 1);
