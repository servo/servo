// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id.case
// - src/dstr-binding/default/for-of-var.template
/*---
description: Binding as specified via property name and identifier (for-of statement)
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var iterCount = 0;

for (var { x: y } of [{ x: 23 }]) {
  assert.sameValue(y, 23);
  assert.throws(ReferenceError, function() {
    x;
  });

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
