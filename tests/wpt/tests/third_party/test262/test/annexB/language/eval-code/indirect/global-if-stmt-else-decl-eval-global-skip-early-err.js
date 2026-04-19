// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-skip-early-err.case
// - src/annex-b-fns/eval-global/indirect-if-stmt-else-decl.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (IfStatement with a declaration in the second statement position in eval code)
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
---*/

(0,eval)(
  'let f = 123;\
  assert.sameValue(f, 123, "binding is not initialized to `undefined`");if (false) ; else function f() {  }assert.sameValue(f, 123, "value is not updated following evaluation");'
);
