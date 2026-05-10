// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8.5
description: Object reference in expression position of TemplateMiddleList
info: |
    TemplateMiddleList : TemplateMiddle Expression

    1. Let head be the TV of TemplateMiddle as defined in 11.8.6.
    2. Let sub be the result of evaluating Expression.
    3. Let middle be ToString(sub).
    4. ReturnIfAbrupt(middle).
---*/

var plain = {};
var custom = {
  toString: function() {
    return '"own" toString';
  }
};

assert.sameValue(`${0} ${plain}`, '0 [object Object]');
assert.sameValue(`${0} ${plain}`, '0 [object Object]');
assert.sameValue(`${0} ${plain}2`, '0 [object Object]2');
assert.sameValue(`${0} ${plain}2`, '0 [object Object]2');

assert.sameValue(`${0} ${custom}`, '0 "own" toString');
assert.sameValue(`${0} ${custom}`, '0 "own" toString');
assert.sameValue(`${0} ${custom}2`, '0 "own" toString2');
assert.sameValue(`${0} ${custom}2`, '0 "own" toString2');
