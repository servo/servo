// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-skip-early-err-switch.case
// - src/annex-b-fns/eval-global/direct-switch-case.template
/*---
description: Extension not observed when creation of variable binding would produce an early error (switch statement) (Function declaration in the `case` clause of a `switch` statement in eval code)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    ii. If replacing the FunctionDeclaration f with a VariableStatement that
        has F as a BindingIdentifier would not produce any Early Errors for
        body, then
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

eval(
  'switch (0) {\
    default:\
      let f;switch (1) {' +
  '  case 1:' +
  '    function f() {  }' +
  '}\
  }'
);

assert.throws(ReferenceError, function() {
  f;
}, 'An initialized binding is not created following evaluation');
assert.sameValue(
  typeof f,
  'undefined',
  'An uninitialized binding is not created following evaluation'
);
