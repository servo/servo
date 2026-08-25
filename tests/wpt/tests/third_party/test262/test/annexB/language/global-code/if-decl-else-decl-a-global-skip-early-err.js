// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-skip-early-err.case
// - src/annex-b-fns/global/if-decl-else-decl-a.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (IfStatement with a declaration in both statement positions in the global scope)
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
    b. If replacing the FunctionDeclaration f with a VariableStatement that has
       F as a BindingIdentifier would not produce any Early Errors for script,
       then
    [...]
---*/
let f = 123;
assert.sameValue(f, 123, 'binding is not initialized to `undefined`');

if (true) function f() {  } else function _f() {}

assert.sameValue(f, 123, 'value is not updated following evaluation');
