// This file was procedurally generated from the following sources:
// - src/function-forms/eval-var-scope-syntax-err.case
// - src/function-forms/error-no-strict/async-gen-named-func-expr.template
/*---
description: sloppy direct eval in params introduces var (async generator named function expression in sloppy code)
esid: sec-asyncgenerator-definitions-evaluation
features: [default-parameters, async-iteration]
flags: [generated, noStrict]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
        [...]


    
    Runtime Semantics: IteratorBindingInitialization
    FormalParameter : BindingElement

    1. Return the result of performing IteratorBindingInitialization for BindingElement with arguments iteratorRecord and environment.

---*/


var callCount = 0;
var f;
f = async function* g(a = eval("var a = 42")) {
  
  callCount = callCount + 1;
};

assert.throws(SyntaxError, function() {
  f();
});
assert.sameValue(callCount, 0, 'generator function body not evaluated');
