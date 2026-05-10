// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: IsRegExp should only be called once
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    1. Let R be the this value.
    [...]
    4. Let C be ? SpeciesConstructor(R, %RegExp%).
    5. Let flags be ? ToString(? Get(R, "flags")).
    6. Let matcher be ? Construct(C, « R, flags »).

  21.2.3.1 RegExp ( pattern, flags )
    1. Let patternIsRegExp be ? IsRegExp(pattern).
    [...]
features: [Symbol.match, Symbol.matchAll]
---*/

var internalCount = 0;
Object.defineProperty(RegExp.prototype, Symbol.match, {
  get: function() {
    ++internalCount;
    return true;
  }
});

var calls = [];
var o = {
  get [Symbol.match]() {
    calls.push('get @@match');
    return false;
  },
  get flags() {
    calls.push('get flags');
    return {
      toString() {
        calls.push('flags toString');
        return "";
      }
    };
  },
};

RegExp.prototype[Symbol.matchAll].call(o, {
  toString() {
    calls.push('arg toString')
  }
});

assert.sameValue(0, internalCount);

assert.sameValue(calls.length, 4);
assert.sameValue(calls[0], 'arg toString');
assert.sameValue(calls[1], 'get flags');
assert.sameValue(calls[2], 'flags toString');
assert.sameValue(calls[3], 'get @@match');
