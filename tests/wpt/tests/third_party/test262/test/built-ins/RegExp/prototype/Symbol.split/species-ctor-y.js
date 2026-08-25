// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: The `y` flag is always used in constructing the "splitter" object
info: |
    [...]
    5. Let C be SpeciesConstructor(rx, %RegExp%).
    [...]
    11. If flags contains "y", let newFlags be flags.
    12. Else, let newFlags be the string that is the concatenation of flags and
        "y".
    13. Let splitter be Construct(C, «rx, newFlags»).
    [...]
features: [Symbol.split, Symbol.species]
---*/

var flagsArg;
var re = {};
re.constructor = function() {};
re.constructor[Symbol.species] = function(_, flags) {
  flagsArg = flags;
  return /./y;
};

re.flags = '';
RegExp.prototype[Symbol.split].call(re, '');
assert.sameValue(flagsArg, 'y');

re.flags = 'abcd';
RegExp.prototype[Symbol.split].call(re, '');
assert.sameValue(flagsArg, 'abcdy');

re.flags = 'Y';
RegExp.prototype[Symbol.split].call(re, '');
assert.sameValue(flagsArg, 'Yy');

re.flags = 'y';
RegExp.prototype[Symbol.split].call(re, '');
assert.sameValue(flagsArg, 'y');

re.flags = 'abycd';
RegExp.prototype[Symbol.split].call(re, '');
assert.sameValue(flagsArg, 'abycd');
