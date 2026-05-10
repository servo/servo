// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-gettemplateobject
description: Templates are cached according to their site
info: >
    1. For each element _e_ of _templateRegistry_, do
      1. If _e_.[[Site]] is the same Parse Node as _templateLiteral_, then
        1. Return _e_.[[Array]].
---*/
var previousObject = null;
var firstObject = null;
function tag(templateObject) {
  previousObject = templateObject;
}

tag`\uc548\ub155`;

assert(previousObject !== null);
firstObject = previousObject;
previousObject = null;

tag`안녕`;

assert(previousObject !== null);
assert(firstObject !== previousObject);
