// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-expr-throws.case
// - src/spread/error/member-expr.template
/*---
description: Spread operator applied to the only argument when evaluation throws (`new` operator)
esid: sec-new-operator-runtime-semantics-evaluation
features: [generators]
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
---*/

assert.throws(Test262Error, function() {
  new function() {}(...function*() { throw new Test262Error(); }());
});
