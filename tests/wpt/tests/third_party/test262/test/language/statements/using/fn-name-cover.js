// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: >
    Assignment of function `name` attribute (CoverParenthesizedExpression)
info: |
    LexicalBinding : BindingIdentifier Initializer

    ...
    3. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let value be NamedEvaluation of Initializer with argument bindingId
includes: [propertyHelper.js]
features: [explicit-resource-management]
---*/

// NOTE: only way to verify is to patch `Function.prototype` so as not to trigger a TypeError from AddDisposableResource
Function.prototype[Symbol.dispose] = function () {}
{
    using xCover = (0, function() {});
    using cover = (function() {});

    assert(xCover.name !== 'xCover');

    assert.sameValue(cover.name, 'cover');
    verifyNotEnumerable(cover, 'name');
    verifyNotWritable(cover, 'name');
    verifyConfigurable(cover, 'name');
}
