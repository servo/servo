// This file was procedurally generated from the following sources:
// - src/spread/mult-literal.case
// - src/spread/default/super-call.template
/*---
description: Spread operator applied to AssignmentExpression following other elements (SuperCall)
esid: sec-super-keyword-runtime-semantics-evaluation
flags: [generated]
info: |
    SuperCall : super Arguments

    1. Let newTarget be GetNewTarget().
    2. If newTarget is undefined, throw a ReferenceError exception.
    3. Let func be GetSuperConstructor().
    4. ReturnIfAbrupt(func).
    5. Let argList be ArgumentListEvaluation of Arguments.
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

class Test262ParentClass {
  constructor() {
    assert.sameValue(arguments.length, 5);
    assert.sameValue(arguments[0], 5);
    assert.sameValue(arguments[1], 6);
    assert.sameValue(arguments[2], 7);
    assert.sameValue(arguments[3], 8);
    assert.sameValue(arguments[4], 9);
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(5, ...[6, 7, 8], 9);
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
