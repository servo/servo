// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Assignment of function `name` attribute (GeneratorExpression)
info: |
    LexicalBinding : BindingIdentifier Initializer

    ...
    3. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let value be NamedEvaluation of Initializer with argument bindingId

flags: [async]
includes: [propertyHelper.js, asyncHelpers.js]
features: [generators,explicit-resource-management]
---*/

// NOTE: only way to verify is to patch `Function.prototype` so as not to trigger a TypeError from AddDisposableResource
Function.prototype[Symbol.dispose] = function () {}
asyncTest(async function() {
    await using xGen = function* x() {};
    await using gen = function*() {};

    assert(xGen.name !== 'xGen');

    assert.sameValue(gen.name, 'gen');
    verifyNotEnumerable(gen, 'name');
    verifyNotWritable(gen, 'name');
    verifyConfigurable(gen, 'name');
});
