// This file was procedurally generated from the following sources:
// - src/spread/mult-expr.case
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
var source = [3, 4, 5];
var target;

var callCount = 0;

class Test262ParentClass {
  constructor() {
    assert.sameValue(arguments.length, 5);
    assert.sameValue(arguments[0], 1);
    assert.sameValue(arguments[1], 2);
    assert.sameValue(arguments[2], 3);
    assert.sameValue(arguments[3], 4);
    assert.sameValue(arguments[4], 5);
    assert.sameValue(target, source);
    callCount += 1;
  }
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(1, 2, ...target = source);
  }
}

new Test262ChildClass();
assert.sameValue(callCount, 1);
