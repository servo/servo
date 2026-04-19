// This file was procedurally generated from the following sources:
// - src/annex-b-fns/func-existing-fn-no-init.case
// - src/annex-b-fns/func/block.template
/*---
description: Existing variable binding is not modified (Block statement in function scope containing a function declaration)
esid: sec-web-compat-functiondeclarationinstantiation
flags: [generated, noStrict]
info: |
    B.3.3.1 Changes to FunctionDeclarationInstantiation

    [...]
    2. If instantiatedVarNames does not contain F, then
    [...]
---*/
var init;

(function() {
  init = f;

  {
    function f() { return 'inner declaration'; }
  }

  function f() {
    return 'outer declaration';
  }
}());

assert.sameValue(init(), 'outer declaration');
