// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-no-skip-try.case
// - src/annex-b-fns/func/block.template
/*---
description: Extension is observed when creation of variable binding would not produce an early error (try statement) (Block statement in function scope containing a function declaration)
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
  assert.sameValue(
    f, undefined, 'Initialized binding created prior to evaluation'
  );

  try {
    throw null;
  } catch (f) {

  {
    function f() { return 123; }
  }

  }

  assert.sameValue(
    typeof f,
    'function',
    'binding value is updated following evaluation'
  );
  assert.sameValue(f(), 123);
}());
