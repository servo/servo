// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Invocation of @@matchAll property of user-supplied RegExp objects
info: |
  String.prototype.matchAll ( regexp )
    [...]
    2. If regexp is neither undefined nor null, then
      a. Let matcher be ? GetMethod(regexp, @@matchAll).
      b. If matcher is not undefined, then
        i. Return ? Call(matcher, regexp, « O »).
features: [Symbol.matchAll, String.prototype.matchAll]
---*/

var obj = {};
var returnVal = {};
var callCount = 0;
var thisVal, args;

obj[Symbol.matchAll] = function () {
  callCount++;
  thisVal = this;
  args = arguments;
  return returnVal;
};

var str = '';

assert.sameValue(str.matchAll(obj), returnVal);
assert.sameValue(callCount, 1, 'Invokes the method exactly once');
assert.sameValue(thisVal, obj);
assert.notSameValue(args, undefined);
assert.sameValue(args.length, 1);
assert.sameValue(args[0], str);
