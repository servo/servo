// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-no-skip-param.case
// - src/annex-b-fns/eval-func/direct-block.template
/*---
description: Extension observed when there is a formal parameter with the same name (Block statement in eval code containing a function declaration)
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
    'init = f;{ function f() {  } }after = f;'
  );
}(123));

assert.sameValue(init, 123, 'binding is not initialized to `undefined`');
assert.sameValue(
  typeof after, 'function', 'value is updated following evaluation'
);
