// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of @@search property of user-supplied objects
es6id: 21.1.3.15
info: |
    [...]
    3. If regexp is neither undefined nor null, then
       a. Let searcher be GetMethod(regexp, @@search).
       b. ReturnIfAbrupt(searcher).
       c. If searcher is not undefined, then
          i. Return Call(searcher, regexp, «O»)
features: [Symbol.search]
---*/

var regexp = {};
var returnVal = {};
var callCount = 0;
var thisVal, args;

regexp[Symbol.search] = function() {
  callCount += 1;
  thisVal = this;
  args = arguments;
  return returnVal;
};

assert.sameValue('O'.search(regexp), returnVal);
assert.sameValue(callCount, 1, 'Invokes the method exactly once');
assert.sameValue(thisVal, regexp);
assert.notSameValue(args, undefined);
assert.sameValue(args.length, 1);
assert.sameValue(args[0], 'O');
