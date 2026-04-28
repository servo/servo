// This file was procedurally generated from the following sources:
// - src/annex-b-fns/global-skip-early-err-for-of.case
// - src/annex-b-fns/global/switch-case.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (for-of statement) (Function declaration in the `case` clause of a `switch` statement in the global scope)
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
  case 1:
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
