// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Assignment of function `name` attribute (FunctionExpression)
info: |
    LexicalBinding : BindingIdentifier Initializer

    ...
    3. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let value be NamedEvaluation of Initializer with argument bindingId

flags: [async]
includes: [propertyHelper.js, asyncHelpers.js]
features: [explicit-resource-management]
---*/

// NOTE: only way to verify is to patch `Function.prototype` so as not to trigger a TypeError from AddDisposableResource
Function.prototype[Symbol.dispose] = function () {}
asyncTest(async function() {
    await using xFn = function x() {};
    await using fn = function() {};

    assert(xFn.name !== 'xFn');

    assert.sameValue(fn.name, 'fn');
    verifyNotEnumerable(fn, 'name');
    verifyNotWritable(fn, 'name');
    verifyConfigurable(fn, 'name');
});
