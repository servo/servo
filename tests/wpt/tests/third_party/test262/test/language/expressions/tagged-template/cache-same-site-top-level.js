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

let templates = [];

function tag(templateObject) {
  templates.push(templateObject);
}

let a = 1;
for (let i = 0; i < 2; i++) {
  tag`head${a}tail`;
}

assert.sameValue(templates.length, 2);

assert.sameValue(
  templates[0],
  templates[1],
  'The realm\'s template cache is for source code locations in a top-level script'
);


