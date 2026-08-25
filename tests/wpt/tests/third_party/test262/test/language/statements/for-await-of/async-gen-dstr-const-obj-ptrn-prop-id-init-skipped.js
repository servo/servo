// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/obj-ptrn-prop-id-init-skipped.case
// - src/dstr-binding-for-await/default/for-await-of-async-gen-const.template
/*---
description: Destructuring initializer is not evaluated when value is not `undefined` (for-await-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding, async-iteration]
flags: [generated, async]
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    BindingElement : BindingPattern Initializer_opt

    [...]
    3. If Initializer is present and v is undefined, then
    [...]
---*/
var initCount = 0;
function counter() {
  initCount += 1;
}

var iterCount = 0;

async function *fn() {
  for await (const { s: t = counter(), u: v = counter(), w: x = counter(), y: z = counter() } of [{ s: null, u: 0, w: false, y: '' }]) {
    assert.sameValue(t, null);
    assert.sameValue(v, 0);
    assert.sameValue(x, false);
    assert.sameValue(z, '');
    assert.sameValue(initCount, 0);

    assert.throws(ReferenceError, function() {
      s;
    });
    assert.throws(ReferenceError, function() {
      u;
    });
    assert.throws(ReferenceError, function() {
      w;
    });
    assert.throws(ReferenceError, function() {
      y;
    });

    iterCount += 1;
  }
}

fn().next()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
