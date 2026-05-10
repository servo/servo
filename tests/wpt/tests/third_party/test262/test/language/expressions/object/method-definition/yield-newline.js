// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Newlines terminate `yield` expressions.
features: [generators]
es6id: 14.4
---*/

var iter, result;
var obj = {
  *g() {
    yield
    1
  }
};

iter = obj.g();

result = iter.next();
assert.sameValue(result.value, undefined, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Second result `value`');
assert.sameValue(result.done, true, 'Second result `done` flag');
