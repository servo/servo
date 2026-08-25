// This file was procedurally generated from the following sources:
// - src/spread/mult-err-itr-step.case
// - src/spread/error/member-expr.template
/*---
description: Spread operator following other arguments when IteratorStep fails (`new` operator)
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

    ArgumentList : ArgumentList , ... AssignmentExpression

    1. Let precedingArgs be the result of evaluating ArgumentList.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let iterator be GetIterator(GetValue(spreadRef) ).
    4. ReturnIfAbrupt(iterator).

    7.4.5 IteratorStep ( iterator )

    1. Let result be IteratorNext(iterator).
    2. ReturnIfAbrupt(result).

    7.4.2 IteratorNext ( iterator, value )

    1. If value was not passed, then
       a. Let result be Invoke(iterator, "next", « »).
    [...]
    3. ReturnIfAbrupt(result).
---*/
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      throw new Test262Error();
    }
  };
};

assert.throws(Test262Error, function() {
  new function() {}(0, ...iter);
});
