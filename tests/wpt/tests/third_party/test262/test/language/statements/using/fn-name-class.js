// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
description: Assignment of function `name` attribute (ClassExpression)
info: |
    LexicalBinding : BindingIdentifier Initializer

    ...
    3. If IsAnonymousFunctionDefinition(Initializer) is true, then
       a. Let value be NamedEvaluation of Initializer with argument bindingId
includes: [propertyHelper.js]
features: [class, explicit-resource-management]
---*/
{
    using xCls = class x { static [Symbol.dispose]() {} };
    using cls = class { static [Symbol.dispose]() {} };
    using xCls2 = class { static name() {} static [Symbol.dispose]() {} };

    assert.notSameValue(xCls.name, 'xCls');
    assert.notSameValue(xCls2.name, 'xCls2');

    assert.sameValue(cls.name, 'cls');
    verifyNotEnumerable(cls, 'name');
    verifyNotWritable(cls, 'name');
    verifyConfigurable(cls, 'name');
}
