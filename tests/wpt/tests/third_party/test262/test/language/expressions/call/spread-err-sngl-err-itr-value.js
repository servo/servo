// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-itr-value.case
// - src/spread/error/call-expr.template
/*---
description: Spread operator applied to the only argument when IteratorValue fails (CallExpression)
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
    6. Repeat
       a. Let next be IteratorStep(iterator).
       b. ReturnIfAbrupt(next).
       c. If next is false, return list.
       d. Let nextArg be IteratorValue(next).
       e. ReturnIfAbrupt(nextArg).

    7.4.4 IteratorValue ( iterResult )

    1. Assert: Type(iterResult) is Object.
    2. Return Get(iterResult, "value").

    7.3.1 Get (O, P)

    [...]
    3. Return O.[[Get]](P, O).
---*/
var iter = {};
var poisonedValue = Object.defineProperty({}, 'value', {
  get: function() {
    throw new Test262Error();
  }
});
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedValue;
    }
  };
};

assert.throws(Test262Error, function() {
  (function() {}(...iter));
});
