// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-existing-fn-no-init.case
// - src/annex-b-fns/func/if-stmt-else-decl.template
/*---
description: Existing variable binding is not modified (IfStatement with a declaration in the second statement position in function scope)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    2. If instantiatedVarNames does not contain F, then
    [...]
---*/
var init;

(function() {
  init = f;

  if (false) ; else function f() { return 'inner declaration'; }

  function f() {
    return 'outer declaration';
  }
}());

assert.sameValue(init(), 'outer declaration');
