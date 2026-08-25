// This file was procedurally generated from the following sources:
// - src/spread/mult-err-expr-throws.case
// - src/spread/error/super-call.template
/*---
description: Spread operator following other arguments when evaluation throws (SuperCall)
esid: sec-super-keyword-runtime-semantics-evaluation
features: [generators]
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

---*/

class Test262ParentClass {
  constructor() {}
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(0, ...function*() { throw new Test262Error(); }());
  }
}

assert.throws(Test262Error, function() {
  new Test262ChildClass();
});
