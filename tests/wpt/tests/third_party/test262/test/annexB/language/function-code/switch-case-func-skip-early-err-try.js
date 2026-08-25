// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-skip-early-err-try.case
// - src/annex-b-fns/func/switch-case.template
/*---
description: Extension is observed when creation of variable binding would not produce an early error (try statement) (Function declaration in the `case` clause of a `switch` statement in function scope)
esid: sec-web-compat-functiondeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    2. If instantiatedVarNames does not contain F, then
       a. Perform ! varEnvRec.CreateMutableBinding(F, false).
       b. Perform varEnvRec.InitializeBinding(F, undefined).
       c. Append F to instantiatedVarNames.
    [...]

    B.3.5 VariableStatements in Catch Blocks

    [...]
    - It is a Syntax Error if any element of the BoundNames of CatchParameter
      also occurs in the VarDeclaredNames of Block unless CatchParameter is
      CatchParameter:BindingIdentifier and that element is only bound by a
      VariableStatement, the VariableDeclarationList of a for statement, or the
      ForBinding of a for-in statement.
---*/

(function() {
  assert.throws(ReferenceError, function() {
    f;
  }, 'An initialized binding is not created prior to evaluation');
  assert.sameValue(
    typeof f,
    'undefined',
    'An uninitialized binding is not created prior to evaluation'
  );

  try {
    throw {};
  } catch ({ f }) {

  switch (1) {
    case 1:
      function f() {  }
  }

  }

  assert.throws(ReferenceError, function() {
    f;
  }, 'An initialized binding is not created following evaluation');
  assert.sameValue(
    typeof f,
    'undefined',
    'An uninitialized binding is not created following evaluation'
  );
}());
