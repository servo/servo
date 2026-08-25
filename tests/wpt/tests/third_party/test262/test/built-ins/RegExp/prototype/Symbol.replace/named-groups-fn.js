// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  "groups" value is passed as last argument of replacer unless it is undefined.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    j. Let namedCaptures be ? Get(result, "groups").
    k. If functionalReplace is true, then
      [...]
      iv. If namedCaptures is not undefined, then
        1. Append namedCaptures as the last element of replacerArgs.
      v. Let replValue be ? Call(replaceValue, undefined, replacerArgs).
features: [Symbol.replace, regexp-named-groups]
---*/

var matchGroups;
var re = /./;
re.exec = function() {
  return {
    length: 1,
    0: "a",
    index: 0,
    groups: matchGroups,
  };
};

var replacerCalls = 0;
var replacerLastArg;
var replacer = function() {
  replacerCalls++;
  replacerLastArg = arguments[arguments.length - 1];
};

matchGroups = null;
re[Symbol.replace]("a", replacer);
assert.sameValue(replacerCalls, 1);
assert.sameValue(replacerLastArg, matchGroups);

matchGroups = undefined;
re[Symbol.replace]("a", replacer);
assert.sameValue(replacerCalls, 2);
assert.sameValue(replacerLastArg, "a");

matchGroups = 10;
re[Symbol.replace]("a", replacer);
assert.sameValue(replacerCalls, 3);
assert.sameValue(replacerLastArg, matchGroups);

matchGroups = {};
re[Symbol.replace]("a", replacer);
assert.sameValue(replacerCalls, 4);
assert.sameValue(replacerLastArg, matchGroups);
