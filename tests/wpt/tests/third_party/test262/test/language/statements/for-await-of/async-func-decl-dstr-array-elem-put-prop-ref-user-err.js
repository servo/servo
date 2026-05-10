// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-elem-put-prop-ref-user-err.case
// - src/dstr-assignment-for-await/default/async-func-decl.template
/*---
description: Any error raised as a result of setting the value should be forwarded to the runtime. (for-await-of statement in an async function declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding, async-iteration]
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
---*/
let x = {
  set y(val) {
    throw new Test262Error();
  }
};

let iterCount = 0;
async function fn() {
  for await ([x.y] of [[23]
]) {
    
    iterCount += 1;
  }
}

let promise = fn();

promise.then(() => $DONE('Promise incorrectly fulfilled.'), ({ constructor }) => {
  assert.sameValue(iterCount, 0);
  assert.sameValue(constructor, Test262Error);
}).then($DONE, $DONE);
