// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-get-err.case
// - src/dstr-binding/error/async-gen-func-named-expr-dflt.template
/*---
description: Abrupt completion returned by GetIterator (async generator named function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [Symbol.iterator, async-iteration]
flags: [generated]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
        [...]

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingPattern : ArrayBindingPattern

    1. Let iterator be GetIterator(value).
    2. ReturnIfAbrupt(iterator).

---*/
var iter = {};
iter[Symbol.iterator] = function() {
  throw new Test262Error();
};


var f;
f = async function* h([x] = iter) {
  
};

assert.throws(Test262Error, function() {
  f();
});
