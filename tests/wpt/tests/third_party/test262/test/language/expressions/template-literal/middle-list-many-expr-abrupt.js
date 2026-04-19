// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8.5
description: Abrupt completion when evaluating expression of TemplateMiddleList
info: |
    TemplateMiddleList : TemplateMiddleList TemplateMiddle Expression

    1. Let rest be the result of evaluating TemplateMiddleList .
    2. ReturnIfAbrupt(rest).
    3. Let middle be the TV of TemplateMiddle as defined in 11.8.6.
    4. Let sub be the result of evaluating Expression.
    5. Let last be ToString(sub).
    6. ReturnIfAbrupt(last).
---*/

assert.throws(Test262Error, function() {
  `${0}${1}${function() { throw new Test262Error(); }()}`;
});
