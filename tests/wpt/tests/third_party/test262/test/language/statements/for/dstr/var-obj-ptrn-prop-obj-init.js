// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-obj-init.case
// - src/dstr-binding/default/for-var.template
/*---
description: Object binding pattern with "nested" object binding pattern using initializer (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated]
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    [...]
    3. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

var iterCount = 0;

for (var { w: { x, y, z } = { x: 4, y: 5, z: 6 } } = { w: undefined }; iterCount < 1; ) {
  assert.sameValue(x, 4);
  assert.sameValue(y, 5);
  assert.sameValue(z, 6);

  assert.throws(ReferenceError, function() {
    w;
  });
  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
