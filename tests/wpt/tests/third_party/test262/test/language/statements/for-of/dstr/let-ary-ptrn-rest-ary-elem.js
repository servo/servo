// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-ary-elem.case
// - src/dstr-binding/default/for-of-let.template
/*---
description: Rest element containing an array BindingElementList pattern (for-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
        for ( ForDeclaration of AssignmentExpression ) Statement

    [...]
    3. Return ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
       lexicalBinding, labelSet).

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
              [...]
          iii. Else,
               1. Assert: lhsKind is lexicalBinding.
               2. Assert: lhs is a ForDeclaration.
               3. Let status be the result of performing BindingInitialization
                  for lhs passing nextValue and iterationEnv as arguments.
          [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingPattern

    1. Let A be ArrayCreate(0).
    [...]
    3. Repeat
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. Return the result of performing BindingInitialization of
             BindingPattern with A and environment as the arguments.
       [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    4. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).
       d. If next is false, set iteratorRecord.[[done]] to true.
       e. Else,
          [...]
          i. Let v be IteratorValue(next).
          ii. If v is an abrupt completion, set
              iteratorRecord.[[done]] to true.
          iii. ReturnIfAbrupt(v).
    5. If iteratorRecord.[[done]] is true, let v be undefined.
    [...]
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var iterCount = 0;

for (let [...[x, y, z]] of [[3, 4, 5]]) {
  assert.sameValue(x, 3);
  assert.sameValue(y, 4);
  assert.sameValue(z, 5);

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
