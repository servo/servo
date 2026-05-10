// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8.5
description: Primitive value in expression position of TemplateMiddleList
info: |
    TemplateMiddleList : TemplateMiddle Expression

    1. Let head be the TV of TemplateMiddle as defined in 11.8.6.
    2. Let sub be the result of evaluating Expression.
    3. Let middle be ToString(sub).
---*/

assert.sameValue(`${0} ${5} bar`, '0 5 bar', 'number value');
assert.sameValue(`${0} ${'string'} bar`, '0 string bar', 'string value');
