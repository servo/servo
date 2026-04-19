// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.fill
description: >
  Fills elements from coerced to Integer `start` and `end` values
info: |
  Array.prototype.fill ( _value_ [ , _start_ [ , _end_ ] ] )

  3. Let _relativeStart_ be ? ToIntegerOrInfinity(_start_).
  4. If _relativeStart_ = -∞, let _k_ be 0.
  5. Else if _relativeStart_ &lt; 0, let _k_ be max(_len_ + _relativeStart_, 0).
  
  7. If _end_ is *undefined*, let _relativeEnd_ be _len_; else let _relativeEnd_ be ? ToIntegerOrInfinity(_end_).
  8. If _relativeEnd_ = -∞, let _final_ be 0.

includes: [compareArray.js]
---*/

assert.compareArray([0, 0].fill(1, undefined), [1, 1],
  '[0, 0].fill(1, undefined) must return [1, 1]'
);

assert.compareArray([0, 0].fill(1, 0, undefined), [1, 1],
  '[0, 0].fill(1, 0, undefined) must return [1, 1]'
);

assert.compareArray([0, 0].fill(1, null), [1, 1],
  '[0, 0].fill(1, null) must return [1, 1]'
);

assert.compareArray([0, 0].fill(1, 0, null), [0, 0],
  '[0, 0].fill(1, 0, null) must return [0, 0]'
);

assert.compareArray([0, 0].fill(1, true), [0, 1],
  '[0, 0].fill(1, true) must return [0, 1]'
);

assert.compareArray([0, 0].fill(1, 0, true), [1, 0],
  '[0, 0].fill(1, 0, true) must return [1, 0]'
);

assert.compareArray([0, 0].fill(1, false), [1, 1],
  '[0, 0].fill(1, false) must return [1, 1]'
);

assert.compareArray([0, 0].fill(1, 0, false), [0, 0],
  '[0, 0].fill(1, 0, false) must return [0, 0]'
);

assert.compareArray([0, 0].fill(1, NaN), [1, 1],
  '[0, 0].fill(1, NaN) must return [1, 1]'
);

assert.compareArray([0, 0].fill(1, 0, NaN), [0, 0],
  '[0, 0].fill(1, 0, NaN) must return [0, 0]'
);

assert.compareArray([0, 0].fill(1, '1'), [0, 1],
  '[0, 0].fill(1, "1") must return [0, 1]'
);

assert.compareArray([0, 0].fill(1, 0, '1'), [1, 0],
  '[0, 0].fill(1, 0, "1") must return [1, 0]'
);

assert.compareArray([0, 0].fill(1, 1.5), [0, 1],
  '[0, 0].fill(1, 1.5) must return [0, 1]'
);

assert.compareArray([0, 0].fill(1, 0, 1.5), [1, 0],
  '[0, 0].fill(1, 0, 1.5) must return [1, 0]'
);

assert.compareArray([0, 0].fill(1, Number.NEGATIVE_INFINITY, 1), [1, 0],
  '[0, 0].fill(1, Number.NEGATIVE_INFINITY, 1) must return [1, 0]'
);

assert.compareArray([0, 0].fill(1, 0, Number.NEGATIVE_INFINITY), [0, 0],
  '[0, 0].fill(1, 0, Number.NEGATIVE_INFINITY) must return [0, 0]'
);
