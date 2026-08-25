// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: |
  Doesn't access the "unicode" property of the constructed RegExp
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    4. Let C be ? SpeciesConstructor(R, %RegExp%).
    5. Let flags be ? ToString(? Get(R, "flags")).
    6. Let matcher be ? Construct(C, « R, flags »).
    [...]
    11. If flags contains "u", let fullUnicode be true.
    12. Else, let fullUnicode be false.
    [...]
features: [Symbol.matchAll, Symbol.species]
---*/

var regexp = /./;
regexp.constructor = {
  [Symbol.species]: function() {
    return Object.defineProperty(/./, 'unicode', {
      get() {
        throw new Test262Error();
      }
    });
  }
};

regexp[Symbol.matchAll]('');
