// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-trailing-comma.case
// - src/dstr-binding/default/for-var.template
/*---
description: Trailing comma is allowed following BindingPropertyList (for statement)
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

    13.3.3 Destructuring Binding Patterns

    ObjectBindingPattern[Yield] :
        { }
        { BindingPropertyList[?Yield] }
        { BindingPropertyList[?Yield] , }
---*/

var iterCount = 0;

for (var { x: y, } = { x: 23 }; iterCount < 1; ) {
  assert.sameValue(y, 23);

  assert.throws(ReferenceError, function() {
    x;
  });
  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
