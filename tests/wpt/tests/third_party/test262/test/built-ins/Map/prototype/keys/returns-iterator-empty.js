// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.keys
description: >
  Returns an iterator on an empty Map object.
info: |
  Map.prototype.keys ()

  ...
  2. Return CreateMapIterator(M, "key").

  23.1.5.1 CreateMapIterator Abstract Operation

  ...
  7. Return iterator.
---*/

var map = new Map();
var iterator = map.keys();
var result = iterator.next();

assert.sameValue(
  result.value, undefined,
  'The value of `result.value` is `undefined`'
);
assert.sameValue(result.done, true, 'The value of `result.done` is `true`');
