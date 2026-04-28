// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Arguments of functional replaceValue
es6id: 21.2.5.8
info: |
    16. Repeat, for each result in results,
        [...]
        m. If functionalReplace is true, then
           i. Let replacerArgs be «matched».
           ii. Append in list order the elements of captures to the end of the
               List replacerArgs.
           iii. Append position and S as the last two elements of replacerArgs.
           iv. Let replValue be Call(replaceValue, undefined, replacerArgs).
           [...]
features: [Symbol.replace]
---*/

var args;
var replacer = function() {
  args = arguments;
};

/b(.).(.)/[Symbol.replace]('abcdef', replacer);

assert.notSameValue(args, undefined);
assert.sameValue(args.length, 5);
assert.sameValue(args[0], 'bcde');
assert.sameValue(args[1], 'c');
assert.sameValue(args[2], 'e');
assert.sameValue(args[3], 1);
assert.sameValue(args[4], 'abcdef');
