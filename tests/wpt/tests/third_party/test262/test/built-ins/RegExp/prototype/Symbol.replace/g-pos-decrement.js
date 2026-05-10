// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when position is decremented during result accumulation
es6id: 21.2.5.8
info: |
    16. Repeat, for each result in results,
        [...]
        p. If position ≥ nextSourcePosition, then
           i. NOTE position should not normally move backwards. If it does, it
              is an indication of an ill-behaving RegExp subclass or use of an
              access triggered side-effect to change the global flag or other
              characteristics of rx. In such cases, the corresponding
              substitution is ignored.
           ii. Let accumulatedResult be the String formed by concatenating the
               code units of the current value of accumulatedResult with the
               substring of S consisting of the code units from
               nextSourcePosition (inclusive) up to position (exclusive) and
               with the code units of replacement.
           iii. Let nextSourcePosition be position + matchLength.
features: [Symbol.replace]
---*/

var r = /./g;
var callCount = 0;
r.exec = function() {
  callCount += 1;

  if (callCount === 1) {
    return { index: 3, length: 1, 0: 0 };
  } else if (callCount === 2) {
    return { index: 1, length: 1, 0: 0 };
  }

  return null;
};

assert.sameValue(r[Symbol.replace]('abcde', 'X'), 'abcXe');
assert.sameValue(callCount, 3);
