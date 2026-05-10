// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-skip-param.case
// - src/annex-b-fns/func/switch-dflt.template
/*---
description: Extension not observed when there is a formal parameter with the same name (Funtion declaration in the `default` clause of a `switch` statement in function scope)
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

  switch (1) {
    default:
      function f() {  }
  }

  after = f;
}(123));

assert.sameValue(init, 123, 'binding is not initialized to `undefined`');
assert.sameValue(after, 123, 'value is not updated following evaluation');
