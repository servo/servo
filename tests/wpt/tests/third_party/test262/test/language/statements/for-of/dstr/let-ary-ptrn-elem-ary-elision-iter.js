// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-ary-elision-iter.case
// - src/dstr-binding/default/for-of-let.template
/*---
description: BindingElement with array binding pattern and initializer is not used (for-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding]
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

    BindingElement : BindingPatternInitializer opt

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       [...]
       e. Else,
          i. Let v be IteratorValue(next).
          [...]
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.

---*/
var callCount = 0;
function* g() {
  callCount += 1;
};

var iterCount = 0;

for (let [[,] = g()] of [[[]]]) {
  assert.sameValue(callCount, 0);

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
