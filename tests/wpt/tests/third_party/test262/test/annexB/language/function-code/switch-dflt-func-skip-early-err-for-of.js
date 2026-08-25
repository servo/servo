// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-skip-early-err-for-of.case
// - src/annex-b-fns/func/switch-dflt.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (for-of statement) (Funtion declaration in the `default` clause of a `switch` statement in function scope)
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

(function() {
  assert.throws(ReferenceError, function() {
    f;
  }, 'An initialized binding is not created prior to evaluation');
  assert.sameValue(
    typeof f,
    'undefined',
    'An uninitialized binding is not created prior to evaluation'
  );

  for (let f of [0]) {

  switch (1) {
    default:
      function f() {  }
  }

  }

  assert.throws(ReferenceError, function() {
    f;
  }, 'An initialized binding is not created following evaluation');
  assert.sameValue(
    typeof f,
    'undefined',
    'An uninitialized binding is not created following evaluation'
  );
}());
