// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-func-existing-var-no-init.case
// - src/annex-b-fns/eval-func/direct-if-decl-no-else.template
/*---
description: Existing variable binding is not modified (IfStatement without an else clause in eval code)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    a. If declaredFunctionOrVarNames does not contain F, then
    [...]
---*/
var init;

(function() {
  eval(
    'var f = 123;\
    init = f;if (true) function f() {  }'
  );
}());

assert.sameValue(init, 123);
