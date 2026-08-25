// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-exhausted.case
// - src/dstr-binding/default/for-of-const.template
/*---
description: RestElement applied to an exhausted iterator (for-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding]
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
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. If environment is undefined, return PutValue(lhs, A).
          ii. Return InitializeReferencedBinding(lhs, A).

---*/

var iterCount = 0;

for (const [, , ...x] of [[1, 2]]) {
  assert(Array.isArray(x));
  assert.sameValue(x.length, 0);

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'iteration occurred as expected');
