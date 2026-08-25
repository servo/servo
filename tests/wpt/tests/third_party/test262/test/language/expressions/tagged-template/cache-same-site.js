// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-gettemplateobject
description: Templates are cached by source location inside a function
info: >
    1. For each element _e_ of _templateRegistry_, do
      1. If _e_.[[Site]] is the same Parse Node as _templateLiteral_, then
        1. Return _e_.[[Array]].
---*/
function tag(templateObject) {
  previousObject = templateObject;
}

var a = 1;
var firstObject = null;
var previousObject = null;

function runTemplate() {
  tag`head${a}tail`;
}

runTemplate();
firstObject = previousObject;

assert(firstObject !== null);
previousObject = null;

runTemplate();

assert.sameValue(
  previousObject,
  firstObject,
  'The realm\'s template cache is for source code locations in a function'
);

