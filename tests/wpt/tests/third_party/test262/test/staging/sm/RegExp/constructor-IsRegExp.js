// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  RegExp constructor with pattern with @@match.
info: bugzilla.mozilla.org/show_bug.cgi?id=1147817
esid: pending
---*/

var matchValue;
var constructorValue;

var matchGet;
var constructorGet;
var sourceGet;
var flagsGet;
function reset() {
  matchGet = false;
  constructorGet = false;
  sourceGet = false;
  flagsGet = false;
}
var obj = {
  get [Symbol.match]() {
    matchGet = true;
    return matchValue;
  },
  get constructor() {
    constructorGet = true;
    return constructorValue;
  },
  get source() {
    sourceGet = true;
    return "foo";
  },
  get flags() {
    flagsGet = true;
    return "i";
  },
  toString() {
    return "bar";
  }
};

matchValue = true;
constructorValue = function() {};

reset();
assert.sameValue(RegExp(obj).toString(), "/foo/i");
assert.sameValue(matchGet, true);
assert.sameValue(constructorGet, true);
assert.sameValue(sourceGet, true);
assert.sameValue(flagsGet, true);

reset();
assert.sameValue(RegExp(obj, "g").toString(), "/foo/g");
assert.sameValue(matchGet, true);
assert.sameValue(constructorGet, false);
assert.sameValue(sourceGet, true);
assert.sameValue(flagsGet, false);

matchValue = false;
constructorValue = function() {};

reset();
assert.sameValue(RegExp(obj).toString(), "/bar/");
assert.sameValue(matchGet, true);
assert.sameValue(constructorGet, false);
assert.sameValue(sourceGet, false);
assert.sameValue(flagsGet, false);

reset();
assert.sameValue(RegExp(obj, "g").toString(), "/bar/g");
assert.sameValue(matchGet, true);
assert.sameValue(constructorGet, false);
assert.sameValue(sourceGet, false);
assert.sameValue(flagsGet, false);

matchValue = true;
constructorValue = RegExp;

reset();
assert.sameValue(RegExp(obj), obj);
assert.sameValue(matchGet, true);
assert.sameValue(constructorGet, true);
assert.sameValue(sourceGet, false);
assert.sameValue(flagsGet, false);
