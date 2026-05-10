// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/for-var.template
/*---
description: Rest object contains just unextracted data (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
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
---*/

var iterCount = 0;

for (var {a, b, ...rest} = {x: 1, y: 2, a: 5, b: 3}; iterCount < 1; ) {
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

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
