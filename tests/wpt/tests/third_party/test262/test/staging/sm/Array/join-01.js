/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.prototype.join
esid: pending
---*/

var count;
var stringifyCounter = { toString: function() { count++; return "obj"; } };

var arr = [1, 2, 3, 4, 5];
assert.sameValue(arr.join(), "1,2,3,4,5");
assert.sameValue(arr.join(","), "1,2,3,4,5");
assert.sameValue(arr.join(undefined), "1,2,3,4,5");
assert.sameValue(arr.join(4), "142434445");
assert.sameValue(arr.join(""), "12345");

count = 0;
assert.sameValue(arr.join(stringifyCounter), "1obj2obj3obj4obj5");
assert.sameValue(count, 1);

var holey = [1, 2, , 4, 5];
assert.sameValue(holey.join(), "1,2,,4,5");
assert.sameValue(holey.join(","), "1,2,,4,5");
assert.sameValue(holey.join(undefined), "1,2,,4,5");
assert.sameValue(holey.join(4), "14244445");

count = 0;
assert.sameValue(holey.join(stringifyCounter), "1obj2objobj4obj5");
assert.sameValue(count, 1);

var nully = [1, 2, 3, null, 5];
assert.sameValue(nully.join(), "1,2,3,,5");
assert.sameValue(nully.join(","), "1,2,3,,5");
assert.sameValue(nully.join(undefined), "1,2,3,,5");
assert.sameValue(nully.join(4), "14243445");

count = 0;
assert.sameValue(nully.join(stringifyCounter), "1obj2obj3objobj5");
assert.sameValue(count, 1);

var undefiney = [1, undefined, 3, 4, 5];
assert.sameValue(undefiney.join(), "1,,3,4,5");
assert.sameValue(undefiney.join(","), "1,,3,4,5");
assert.sameValue(undefiney.join(undefined), "1,,3,4,5");
assert.sameValue(undefiney.join(4), "14434445");

count = 0;
assert.sameValue(undefiney.join(stringifyCounter), "1objobj3obj4obj5");
assert.sameValue(count, 1);

var log = '';
arr = {length: {valueOf: function () { log += "L"; return 2; }},
      0: "x", 1: "z"};
var sep = {toString: function () { log += "S"; return "y"; }};
assert.sameValue(Array.prototype.join.call(arr, sep), "xyz");
assert.sameValue(log, "LS");

var funky =
  {
    toString: function()
    {
      Array.prototype[1] = "chorp";
      Object.prototype[3] = "fnord";
      return "funky";
    }
  };
var trailingHoles = [0, funky, /* 2 */, /* 3 */,];
assert.sameValue(trailingHoles.join(""), "0funkyfnord");
