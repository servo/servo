// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of @@match property of user-supplied objects
es6id: 21.1.3.11
info: |
    [...]
    3. If regexp is neither undefined nor null, then
       a. Let matcher be GetMethod(regexp, @@match).
       b. ReturnIfAbrupt(matcher).
       c. If matcher is not undefined, then
          i. Return Call(matcher, regexp, «O»).
features: [Symbol.match]
---*/

var obj = {};
var returnVal = {};
var callCount = 0;
var thisVal, args;

obj[Symbol.match] = function() {
  callCount += 1;
  thisVal = this;
  args = arguments;
  return returnVal;
};

assert.sameValue(''.match(obj), returnVal);
assert.sameValue(callCount, 1, 'Invokes the method exactly once');
assert.sameValue(thisVal, obj);
assert.notSameValue(args, undefined);
assert.sameValue(args.length, 1);
assert.sameValue(args[0], '');
