// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-no-skip-param.case
// - src/annex-b-fns/eval-func/direct-if-stmt-else-decl.template
/*---
description: Extension observed when there is a formal parameter with the same name (IfStatement with a declaration in the second statement position in eval code)
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
var init, after;

(function(f) {
  eval(
    'init = f;if (false) ; else function f() {  }after = f;'
  );
}(123));

assert.sameValue(init, 123, 'binding is not initialized to `undefined`');
assert.sameValue(
  typeof after, 'function', 'value is updated following evaluation'
);
