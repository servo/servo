// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-skip-early-err.case
// - src/annex-b-fns/global/block.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (Block statement in the global scope containing a function declaration)
esid: sec-web-compat-globaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.2 Changes to GlobalDeclarationInstantiation

    [...]
    b. If replacing the FunctionDeclaration f with a VariableStatement that has
       F as a BindingIdentifier would not produce any Early Errors for script,
       then
    [...]
---*/
let f = 123;
assert.sameValue(f, 123, 'binding is not initialized to `undefined`');

{
  function f() {  }
}

assert.sameValue(f, 123, 'value is not updated following evaluation');
