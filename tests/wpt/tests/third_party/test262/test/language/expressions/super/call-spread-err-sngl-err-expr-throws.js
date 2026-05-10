// This file was procedurally generated from the following sources:
// - src/spread/sngl-err-expr-throws.case
// - src/spread/error/super-call.template
/*---
description: Spread operator applied to the only argument when evaluation throws (SuperCall)
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

    ArgumentList : ... AssignmentExpression

    1. Let list be an empty List.
    2. Let spreadRef be the result of evaluating AssignmentExpression.
    3. Let spreadObj be GetValue(spreadRef).
    4. Let iterator be GetIterator(spreadObj).
    5. ReturnIfAbrupt(iterator).
---*/

class Test262ParentClass {
  constructor() {}
}

class Test262ChildClass extends Test262ParentClass {
  constructor() {
    super(...function*() { throw new Test262Error(); }());
  }
}

assert.throws(Test262Error, function() {
  new Test262ChildClass();
});
