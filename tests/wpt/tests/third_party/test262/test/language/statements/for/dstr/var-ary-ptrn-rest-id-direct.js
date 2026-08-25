// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-direct.case
// - src/dstr-binding/default/for-var.template
/*---
description: Lone rest element (direct binding) (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated]
includes: [compareArray.js]
info: |
    IterationStatement :
        for ( var VariableDeclarationList ; Expressionopt ; Expressionopt ) Statement

    1. Let varDcl be the result of evaluating VariableDeclarationList.
    [...]

    13.3.2.4 Runtime Semantics: Evaluation

    VariableDeclarationList : VariableDeclarationList , VariableDeclaration

    1. Let next be the result of evaluating VariableDeclarationList.
    2. ReturnIfAbrupt(next).
    3. Return the result of evaluating VariableDeclaration.

    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for BindingPattern
       passing rval and undefined as arguments.

    Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingIdentifier

    [...]
    2. Let A be ! ArrayCreate(0).
    3. Let n be 0.
    4. Repeat,
        [...]
        f. Perform ! CreateDataPropertyOrThrow(A, ! ToString(n), nextValue).
        g. Set n to n + 1.

---*/

var iterCount = 0;

for (var [...x] = [1]; iterCount < 1; ) {
  assert(Array.isArray(x));
  assert.compareArray(x, [1]);
  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
