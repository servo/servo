// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-existing-block-fn-no-init.case
// - src/annex-b-fns/global/if-stmt-else-decl.template
/*---
description: Does not re-initialize binding created by similar forms (IfStatement with a declaration in the second statement position in the global scope)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.2 Changes to GlobalDeclarationInstantiation

    [...]
    b. If declaredFunctionOrVarNames does not contain F, then
       i. Perform ? envRec.CreateGlobalFunctionBinding(F, undefined, false).
       ii. Append F to declaredFunctionOrVarNames.
---*/
assert.sameValue(f, undefined);

{
  function f() {}
}

if (false) ; else function f() {  }
