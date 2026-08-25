// Copyright (C) 2018 Igalia, S. L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-gettemplateobject
description: Templates are cached by source location inside a function
info: >
    Each time eval is called, it is a different site. However, a loop within
    the eval is considered the same site. This is a regression test for an
    issue that Caitlin Potter faced in implementations of the new template
    caching semantics in both V8 and JSC.

    1. For each element _e_ of _templateRegistry_, do
      1. If _e_.[[Site]] is the same Parse Node as _templateLiteral_, then
        1. Return _e_.[[Array]].
---*/

let objs = [];
function tag(templateObject) {
  objs.push(templateObject);
}

for (let a = 0; a < 2; a++) {
  eval("\
    (function() {\
      for (let b = 0; b < 2; b++) {\
        tag`${a}${b}`;\
      }\
    })();\
  ");
}

assert.sameValue(objs[0], objs[1]);
assert.notSameValue(objs[1], objs[2]);
assert.sameValue(objs[2], objs[3]);

