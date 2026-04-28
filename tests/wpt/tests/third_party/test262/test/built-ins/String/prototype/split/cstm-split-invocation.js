// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of @@split property of user-supplied objects
es6id: 21.1.3.17
info: |
    [...]
    3. If separator is neither undefined nor null, then
       a. Let splitter be GetMethod(separator, @@split).
       b. ReturnIfAbrupt(splitter).
       c. If splitter is not undefined, then
          i. Return Call(splitter, separator, «O, limit»).
features: [Symbol.split]
---*/

var separator = {};
var returnVal = {};
var callCount = 0;
var thisVal, args;

separator[Symbol.split] = function() {
  callCount += 1;
  thisVal = this;
  args = arguments;
  return returnVal;
};

assert.sameValue(''.split(separator, 'limit'), returnVal);
assert.sameValue(thisVal, separator);
assert.notSameValue(args, undefined);
assert.sameValue(args.length, 2);
assert.sameValue(args[0], '');
assert.sameValue(args[1], 'limit');
