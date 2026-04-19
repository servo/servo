// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-no-skip-param.case
// - src/annex-b-fns/eval-func/direct-switch-dflt.template
/*---
description: Extension observed when there is a formal parameter with the same name (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
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
var init, after;

(function(f) {
  eval(
    'init = f;switch (1) {' +
    '  default:' +
    '    function f() {  }' +
    '}\
    after = f;'
  );
}(123));

assert.sameValue(init, 123, 'binding is not initialized to `undefined`');
assert.sameValue(
  typeof after, 'function', 'value is updated following evaluation'
);
