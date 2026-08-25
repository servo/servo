// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-itr-get-call.case
// - src/spread/error/member-expr.template
/*---
description: Spread operator applied to the only argument when GetIterator fails (@@iterator function invocation) (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [Symbol.iterator]
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

    ArgumentList : ... AssignmentExpression

    1. Let list be an empty List.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let spreadObj be GetValue(spreadRef).
    4. Let iterator be GetIterator(spreadObj).
    5. ReturnIfAbrupt(iterator).

    7.4.1 GetIterator ( obj, method )

    [...]
    3. Let iterator be Call(method,obj).
    4. ReturnIfAbrupt(iterator).
---*/
var iter = {};
iter[Symbol.iterator] = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  new function() {}(...iter);
});
