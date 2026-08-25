// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-existing-block-fn-no-init.case
// - src/annex-b-fns/eval-func/direct-switch-dflt.template
/*---
description: Does not re-initialize binding created by similar forms (Funtion declaration in the `default` clause of a `switch` statement in eval code in the global scope)
esid: sec-web-compat-evaldeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
    [...]
---*/
var init;

(function() {
  eval(
    'init = f;\
    \
    {\
      function f() {}\
    }switch (1) {' +
    '  default:' +
    '    function f() {  }' +
    '}\
    '
  );
}());

assert.sameValue(init, undefined);
