// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-no-skip-try.case
// - src/annex-b-fns/eval-func/direct-block.template
/*---
description: Extension is observed when creation of variable binding would not produce an early error (try statement) (Block statement in eval code containing a function declaration)
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

(function() {
  eval(
    'assert.sameValue(\
      f, undefined, "Initialized binding created prior to evaluation"\
    );\
    \
    try {\
      throw null;\
    } catch (f) {{ function f() { return 123; } }}\
    \
    assert.sameValue(\
      typeof f,\
      "function",\
      "binding value is updated following evaluation"\
    );\
    assert.sameValue(f(), 123);'
  );
}());
