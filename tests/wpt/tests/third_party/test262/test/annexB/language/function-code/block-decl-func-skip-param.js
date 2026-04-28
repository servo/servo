// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-skip-param.case
// - src/annex-b-fns/func/block.template
/*---
description: Extension not observed when there is a formal parameter with the same name (Block statement in function scope containing a function declaration)
esid: sec-web-compat-functiondeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    ii. If replacing the FunctionDeclaration f with a VariableStatement that
        has F as a BindingIdentifier would not produce any Early Errors for
        func and F is not an element of BoundNames of argumentsList, then
    [...]
---*/
var init, after;

(function(f) {
  init = f;

  {
    function f() {  }
  }

  after = f;
}(123));

assert.sameValue(init, 123, 'binding is not initialized to `undefined`');
assert.sameValue(after, 123, 'value is not updated following evaluation');
