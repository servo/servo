// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-get-err-array-prototype.case
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

    Runtime Semantics: BindingInitialization

    BindingPattern : ArrayBindingPattern

    1. Let iteratorRecord be ? GetIterator(value).

    GetIterator ( obj [ , hint [ , method ] ] )

    [...]
    4. Let iterator be ? Call(method, obj).

    Call ( F, V [ , argumentsList ] )

    [...]
    2. If IsCallable(F) is false, throw a TypeError exception.

---*/
delete Array.prototype[Symbol.iterator];


async function* f([x, y, z]) {
  
};

assert.throws(TypeError, function() {
  f([1, 2, 3]);
});
