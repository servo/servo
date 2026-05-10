// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8.5
description: Primitive value in expression position of TemplateLiteral
info: |
    TemplateLiteral : TemplateHead Expression TemplateSpans

    1. Let head be the TV of TemplateHead as defined in 11.8.6.
    2. Let sub be the result of evaluating Expression.
    3. Let middle be ToString(sub).
---*/

assert.sameValue(`foo ${5} bar`, 'foo 5 bar', 'number value');
assert.sameValue(`foo ${'string'} bar`, 'foo string bar', 'string value');
