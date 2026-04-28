// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invocation of @@replace property of user-supplied objects
es6id: 21.1.3.14
info: |
    [...]
    3. If searchValue is neither undefined nor null, then
       a. Let replacer be GetMethod(searchValue, @@replace).
       b. ReturnIfAbrupt(replacer).
       c. If replacer is not undefined, then
          i. Return Call(replacer, searchValue, «O, replaceValue»).
features: [Symbol.replace]
---*/

var searchValue = {};
var returnVal = {};
var callCount = 0;
var thisVal, args;

searchValue[Symbol.replace] = function() {
  callCount += 1;
  thisVal = this;
  args = arguments;
  return returnVal;
};

assert.sameValue(''.replace(searchValue, 'replace value'), returnVal);
assert.sameValue(thisVal, searchValue);
assert.notSameValue(args, undefined);
assert.sameValue(args.length, 2);
assert.sameValue(args[0], '');
assert.sameValue(args[1], 'replace value');
