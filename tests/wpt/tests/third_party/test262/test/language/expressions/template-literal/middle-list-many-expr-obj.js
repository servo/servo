// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.8.5
description: Object reference in expression position of TemplateMiddleList
info: |
    TemplateMiddleList : TemplateMiddleList TemplateMiddle Expression

    1. Let rest be the result of evaluating TemplateMiddleList .
    2. ReturnIfAbrupt(rest).
    3. Let middle be the TV of TemplateMiddle as defined in 11.8.6.
    4. Let sub be the result of evaluating Expression.
    5. Let last be ToString(sub).
---*/

var plain = {};
var custom = {
  toString: function() {
    return '"own" toString';
  }
};

assert.sameValue(`${0} ${1} ${plain}`, '0 1 [object Object]');
assert.sameValue(`${0} ${1} ${plain}`, '0 1 [object Object]');
assert.sameValue(`${0} ${1} ${plain}2`, '0 1 [object Object]2');
assert.sameValue(`${0} ${1} ${plain}2`, '0 1 [object Object]2');

assert.sameValue(`${0} ${1} ${custom}`, '0 1 "own" toString');
assert.sameValue(`${0} ${1} ${custom}`, '0 1 "own" toString');
assert.sameValue(`${0} ${1} ${custom}2`, '0 1 "own" toString2');
assert.sameValue(`${0} ${1} ${custom}2`, '0 1 "own" toString2');
