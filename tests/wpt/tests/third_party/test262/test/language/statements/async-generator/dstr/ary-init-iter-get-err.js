// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-get-err.case
// - src/dstr-binding/error/async-gen-func-decl.template
/*---
description: Abrupt completion returned by GetIterator (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [Symbol.iterator, async-iteration]
flags: [generated]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
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


async function* f([x]) {
  
};

assert.throws(Test262Error, function() {
  f(iter);
});
