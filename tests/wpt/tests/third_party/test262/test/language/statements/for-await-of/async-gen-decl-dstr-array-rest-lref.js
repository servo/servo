// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-rest-lref.case
// - src/dstr-assignment-for-await/async-generator/async-gen-decl.template
/*---
description: Reference is evaluated during assignment (for-await-of statement in an async generator declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
      for await ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    5. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]

    ArrayAssignmentPattern : [ Elisionopt AssignmentRestElement ]

    [...]
    4. Let result be the result of performing
      IteratorDestructuringAssignmentEvaluation of AssignmentRestElement
      with iteratorRecord as the argument.
    5. If iteratorRecord.[[Done]] is false, return ? IteratorClose(iterator, result).


    AssignmentRestElement[Yield] : ... DestructuringAssignmentTarget

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an
       ArrayLiteral, then
       a. Let lref be the result of evaluating DestructuringAssignmentTarget.
       b. ReturnIfAbrupt(lref).
    [...]

---*/
let nextCount = 0;
let returnCount = 0;
let iterator = {
  next() {
    nextCount += 1;
    return { done: true };
  },
  return() {
    returnCount += 1;
  }
};
let obj = {};
let iterable = {
  [Symbol.iterator]() {
    return iterator;
  }
};

let iterCount = 0;
async function * fn() {
  for await ([...obj['a' + 'b']] of [iterable]) {
    assert.sameValue(nextCount, 1);
    assert.sameValue(returnCount, 0);
    assert(!!obj.ab);
    assert.sameValue(obj.ab.length, 0);

    iterCount += 1;
  }
}

let iter = fn();

iter.next()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);

