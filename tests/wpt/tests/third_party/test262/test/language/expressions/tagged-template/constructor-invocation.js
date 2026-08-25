// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.3.7
description: >
    Tagged template application takes precedence over `new` invocation.
---*/

function Constructor(x) {
  arg = x;
}
var tag = function(x) {
  templateObject = x;
  return Constructor;
};
var arg = null;
var instance, templateObject;

instance = new tag`first template`;

assert(instance instanceof Constructor);
assert.sameValue(templateObject[0], 'first template');
assert.sameValue(arg, undefined);

instance = new tag`second template`('constructor argument');
assert.sameValue(templateObject[0], 'second template', 'tagging function');
assert.sameValue(arg, 'constructor argument');
