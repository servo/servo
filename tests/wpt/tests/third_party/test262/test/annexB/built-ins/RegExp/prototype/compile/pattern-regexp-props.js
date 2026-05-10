// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: >
    Properties are not referenced when provided pattern is a RegExp instance
info: |
    [...]
    3. If Type(pattern) is Object and pattern has a [[RegExpMatcher]] internal
       slot, then
       a. If flags is not undefined, throw a TypeError exception.
       b. Let P be the value of pattern's [[OriginalSource]] internal slot.
       c. Let F be the value of pattern's [[OriginalFlags]] internal slot.
    4. Else,
       [...]
    5. Return ? RegExpInitialize(O, P, F).
---*/

var thisValue = /abc/gim;
var pattern = /def/mig;
var flagsCount  = 0;
var globalCount = 0;
var ignoreCaseCount = 0;
var multilineCount = 0;
var stickyCount = 0;
var unicodeCount = 0;
var counters = {
  flags: {
    get: function() {
      flagsCount += 1;
    }
  },
  global: {
    get: function() {
      globalCount += 1;
    }
  },
  ignoreCase: {
    get: function() {
      ignoreCaseCount += 1;
    }
  },
  multiline: {
    get: function() {
      multilineCount += 1;
    }
  },
  sticky: {
    get: function() {
      stickyCount += 1;
    }
  },
  unicode: {
    get: function() {
      unicodeCount += 1;
    }
  }
};

Object.defineProperties(thisValue, counters);
Object.defineProperties(pattern, counters);

thisValue.compile(thisValue);
thisValue.compile(pattern);
thisValue.compile(thisValue);

assert.sameValue(flagsCount, 0, '`flags` property not accessed');
assert.sameValue(globalCount, 0, '`global` property not accessed');
assert.sameValue(ignoreCaseCount, 0, '`ignoreCase` property not accessed');
assert.sameValue(multilineCount, 0, '`multiline` property not accessed');
assert.sameValue(stickyCount, 0, '`sticky` property not accessed');
assert.sameValue(unicodeCount, 0, '`unicode` property not accessed');
