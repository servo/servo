// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-no-skip-try.case
// - src/annex-b-fns/eval-func/direct-if-decl-no-else.template
/*---
description: Extension is observed when creation of variable binding would not produce an early error (try statement) (IfStatement without an else clause in eval code)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


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
    } catch (f) {if (true) function f() { return 123; }}\
    \
    assert.sameValue(\
      typeof f,\
      "function",\
      "binding value is updated following evaluation"\
    );\
    assert.sameValue(f(), 123);'
  );
}());
