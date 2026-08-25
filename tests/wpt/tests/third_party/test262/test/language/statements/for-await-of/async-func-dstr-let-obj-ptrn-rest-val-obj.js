// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/obj-ptrn-rest-val-obj.case
// - src/dstr-binding-for-await/default/for-await-of-async-func-let.template
/*---
description: Rest object contains just unextracted data (for-await-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    IterationStatement :
        for await ( ForDeclaration of AssignmentExpression ) Statement

    [...]
    2. Return ? ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
        lexicalBinding, labelSet, async).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. Let destructuring be IsDestructuring of lhs.
    [...]
    6. Repeat
       [...]
       j. If destructuring is false, then
          [...]
       k. Else
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
---*/

var iterCount = 0;

async function fn() {
  for await (let {a, b, ...rest} of [{x: 1, y: 2, a: 5, b: 3}]) {
    assert.sameValue(rest.a, undefined);
    assert.sameValue(rest.b, undefined);

    verifyProperty(rest, "x", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 1
    });

    verifyProperty(rest, "y", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 2
    });

    iterCount += 1;
  }
}

fn()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);

