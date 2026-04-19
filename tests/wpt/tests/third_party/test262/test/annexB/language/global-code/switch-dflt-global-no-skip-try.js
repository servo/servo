// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-no-skip-try.case
// - src/annex-b-fns/global/switch-dflt.template
/*---
description: Extension is observed when creation of variable binding would not produce an early error (try statement) (Funtion declaration in the `default` clause of a `switch` statement in the global scope)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation

    [...]
    b. If replacing the FunctionDeclaration f with a VariableStatement that has
       F as a BindingIdentifier would not produce any Early Errors for script,
       then
    [...]

    B.3.5 VariableStatements in Catch Blocks

    [...]
    - It is a Syntax Error if any element of the BoundNames of CatchParameter
      also occurs in the VarDeclaredNames of Block unless CatchParameter is
      CatchParameter:BindingIdentifier and that element is only bound by a
      VariableStatement, the VariableDeclarationList of a for statement, or the
      ForBinding of a for-in statement.
---*/
assert.sameValue(
  f, undefined, 'Initialized binding created prior to evaluation'
);

try {
  throw null;
} catch (f) {

switch (1) {
  default:
    function f() { return 123; }
}

}

assert.sameValue(
  typeof f,
  'function',
  'binding value is updated following evaluation'
);
assert.sameValue(f(), 123);
