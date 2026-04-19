// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-existing-block-fn-no-init.case
// - src/annex-b-fns/func/if-decl-no-else.template
/*---
description: Does not re-initialize binding created by similar forms (IfStatement without an else clause in function scope)
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

  {
    function f() {}
  }

  if (true) function f() {  }

  
}());

assert.sameValue(init, undefined);
