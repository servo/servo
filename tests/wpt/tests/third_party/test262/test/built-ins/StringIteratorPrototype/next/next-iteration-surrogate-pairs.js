// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.5.2.1
description: >
    Iteration should respect UTF-16-encoded Unicode code points specified via
    surrogate pairs.
features: [Symbol.iterator]
---*/

var lo = '\uD834';
var hi = '\uDF06';
var pair = lo + hi;
var string = 'a' + pair + 'b' + lo + pair + hi + lo;
var iterator = string[Symbol.iterator]();
var result;

result = iterator.next();
assert.sameValue(result.value, 'a', 'First normal code point `value`');
assert.sameValue(result.done, false, 'First normal code point `done` flag');

result = iterator.next();
assert.sameValue(
  result.value, pair, 'Surrogate pair `value` (between normal code points)'
);
assert.sameValue(
  result.done, false, 'Surrogate pair `done` flag (between normal code points)'
);

result = iterator.next();
assert.sameValue(result.value, 'b', 'Second normal code point `value`');
assert.sameValue(result.done, false, 'Second normal code point `done` flag');

result = iterator.next();
assert.sameValue(
  result.value,
  lo,
  'Lone lower code point `value` (following normal code point)'
);
assert.sameValue(
  result.done,
  false,
  'Lone lower code point `done` flag (following normal code point)'
);

result = iterator.next();
assert.sameValue(
  result.value,
  pair,
  'Surrogate pair `value` (between lone lower- and upper- code points)'
);
assert.sameValue(
  result.done,
  false,
  'Surrogate pair `done` flag (between lone lower- and upper- code points)'
);

result = iterator.next();
assert.sameValue(result.value, hi, 'Lone upper code point `value`');
assert.sameValue(result.done, false, 'Lone upper code point `done` flag');

result = iterator.next();
assert.sameValue(
  result.value,
  lo,
  'Lone lower code point `value` (following lone upper code point)'
);
assert.sameValue(
  result.done,
  false,
  'Lone lower code point `done` flag (following lone upper code point)'
);

result = iterator.next();
assert.sameValue(result.value, undefined, 'Exhausted result `value`');
assert.sameValue(result.done, true, 'Exhausted result `done` flag');

result = iterator.next();
assert.sameValue(
  result.value, undefined, 'Exhausted result `value` (repeated request)'
);
assert.sameValue(
  result.done, true, 'Exhausted result `done` flag (repeated request'
);
