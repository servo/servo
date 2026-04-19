// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  Arguments of functional replaceValue (`exec` result is empty array).
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    e. Let position be ? ToInteger(? Get(result, "index")).
    [...]
    k. If functionalReplace is true, then
      i. Let replacerArgs be « matched ».
      ii. Append in list order the elements of captures to the end of the List replacerArgs.
      iii. Append position and S to replacerArgs.
      [...]
      v. Let replValue be ? Call(replaceValue, undefined, replacerArgs).
features: [Symbol.replace]
---*/

var args;
var replacer = function() {
  args = arguments;
};

var r = /./;
r.exec = function() {
  return [];
};

r[Symbol.replace]('foo', replacer);

assert.notSameValue(args, undefined);
assert.sameValue(args.length, 3);
assert.sameValue(args[0], 'undefined');
assert.sameValue(args[1], 0);
assert.sameValue(args[2], 'foo');
