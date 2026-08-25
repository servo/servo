// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-empty.case
// - src/dstr-binding/default/for-of-var.template
/*---
description: No property access occurs for an "empty" object binding pattern (for-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
        for ( var ForBinding of AssignmentExpression ) Statement

    [...]
    3. Return ForIn/OfBodyEvaluation(ForBinding, Statement, keyResult,
       varBinding, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    3. Let destructuring be IsDestructuring of lhs.
    [...]
    5. Repeat
       [...]
       h. If destructuring is false, then
          [...]
       i. Else
          i. If lhsKind is assignment, then
             [...]
          ii. Else if lhsKind is varBinding, then
              1. Assert: lhs is a ForBinding.
              2. Let status be the result of performing BindingInitialization
                 for lhs passing nextValue and undefined as the arguments.
          [...]

    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/
var accessCount = 0;
var obj = Object.defineProperty({}, 'attr', {
  get: function() {
    accessCount += 1;
  }
});

var iterCount = 0;

for (var {} of [obj]) {
  assert.sameValue(accessCount, 0);

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
