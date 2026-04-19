// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: indexed values are not cached
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
  ...
features: [Array.prototype.includes]
---*/

function getCleanObj() {
  var obj = {};
  Object.defineProperty(obj, "length", {
    get: function() {
      Object.defineProperty(obj, "0", {
        get: function() {
          obj[1] = "ecma262";
          obj[2] = "cake";
          return "tc39";
        }
      });
      return 2;
    }
  });

  return obj;
}

var obj;
obj = getCleanObj();
assert.sameValue([].includes.call(obj, "tc39"), true, "'tc39' is true");

obj = getCleanObj();
assert.sameValue([].includes.call(obj, "ecma262"), true, "'ecma262' is true");

obj = getCleanObj();
assert.sameValue([].includes.call(obj, "cake"), false, "'cake' is false");
assert.sameValue(obj[2], "cake", "'2' is set");
