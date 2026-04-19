// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-skip-early-err-try.case
// - src/annex-b-fns/eval-global/indirect-switch-dflt.template
/*---
description: Extension is not observed when creation of variable binding would produce an early error (try statement) (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    ii. If replacing the FunctionDeclaration f with a VariableStatement that
        has F as a BindingIdentifier would not produce any Early Errors for
        body, then
    [...]

    B.3.5 VariableStatements in Catch Blocks

    [...]
    - It is a Syntax Error if any element of the BoundNames of CatchParameter
      also occurs in the VarDeclaredNames of Block unless CatchParameter is
      CatchParameter:BindingIdentifier and that element is only bound by a
      VariableStatement, the VariableDeclarationList of a for statement, or the
      ForBinding of a for-in statement.
---*/

(0,eval)(
  'assert.throws(ReferenceError, function() {\
    f;\
  }, "An initialized binding is not created prior to evaluation");\
  assert.sameValue(\
    typeof f,\
    "undefined",\
    "An uninitialized binding is not created prior to evaluation"\
  );\
  \
  try {\
    throw {};\
  } catch ({ f }) {switch (1) {' +
  '  default:' +
  '    function f() {  }' +
  '}\
  }\
  \
  assert.throws(ReferenceError, function() {\
    f;\
  }, "An initialized binding is not created following evaluation");\
  assert.sameValue(\
    typeof f,\
    "undefined",\
    "An uninitialized binding is not created following evaluation"\
  );'
);
