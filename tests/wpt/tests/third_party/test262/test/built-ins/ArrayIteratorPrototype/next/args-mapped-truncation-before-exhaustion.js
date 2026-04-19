// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%arrayiteratorprototype%.next
description: >
    Prior to being exhausted, iterators for mapped arguments exotic objects
    should honor argument removal.
flags: [noStrict]
features: [Symbol.iterator]
---*/

(function(a, b, c) {
  var iterator = arguments[Symbol.iterator]();
  var result;

  iterator.next();
  iterator.next();

  arguments.length = 2;

  result = iterator.next();
  assert.sameValue(result.value, undefined, 'Exhausted result `value`');
  assert.sameValue(result.done, true, 'Exhausted result `done` flag');
}(2, 1, 3));
