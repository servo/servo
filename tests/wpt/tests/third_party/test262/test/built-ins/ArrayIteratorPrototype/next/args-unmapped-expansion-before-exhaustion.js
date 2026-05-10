// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%arrayiteratorprototype%.next
description: >
    Prior to being exhausted, iterators for unmapped arguments exotic objects
    should honor the insertion of additional argument values.
flags: [noStrict]
features: [Symbol.iterator]
---*/

(function(a, b, c) {
  'use strict';
  var iterator = arguments[Symbol.iterator]();
  var result;

  iterator.next();
  iterator.next();

  arguments.length = 4;
  arguments[3] = 5;

  result = iterator.next();
  result = iterator.next();
  assert.sameValue(result.value, 5, 'New result `value`');
  assert.sameValue(result.done, false, 'New result `done` flag');

  result = iterator.next();
  assert.sameValue(result.value, undefined, 'Exhausted result `value`');
  assert.sameValue(result.done, true, 'Exhausted result `done` flag');
}(2, 1, 3));
