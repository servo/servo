/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  [].reverse should swap elements low to high using accesses to low elements, then accesses to high elements
info: bugzilla.mozilla.org/show_bug.cgi?id=858677
esid: pending
---*/

var observed = [];

// (0, 7) hits the lowerExists/upperExists case.
// (1, 6) hits the !lowerExists/upperExists case.
// (2, 5) hits the lowerExists/!upperExists case.
// (3, 4) hits the !lowerExists/!upperExists case.
//
// It'd be a good idea to have a second version of this test at some point
// where the "array" being reversed is a proxy, to detect proper ordering of
// getproperty, hasproperty, setproperty into a hole, and deleteproperty from a
// non-configurable element.  But at present our Array.prototype.reverse
// implementation probably doesn't conform fully to all this (because our
// internal MOP is still slightly off), so punt for now.
var props =
  {
    0: {
      configurable: true,
      get: function() { observed.push("index 0 get"); return "index 0 get"; },
      set: function(v) { observed.push("index 0 set: " + v); }
    },
    /* 1: hole */
    2: {
      configurable: true,
      get: function() { observed.push("index 2 get"); return "index 2 get"; },
      set: function(v) { observed.push("index 2 set: " + v); }
    },
    /* 3: hole */
    /* 4: hole */
    /* 5: hole */
    6: {
      configurable: true,
      get: function() { observed.push("index 6 get"); return "index 6 get"; },
      set: function(v) { observed.push("index 6 set: " + v); }
    },
    7: {
      configurable: true,
      get: function() { observed.push("index 7 get"); return "index 7 get"; },
      set: function(v) { observed.push("index 7 set: " + v); }
    },
  };

var arr = Object.defineProperties(new Array(8), props);

arr.reverse();

var expectedObserved =
  ["index 0 get", "index 7 get", "index 0 set: index 7 get", "index 7 set: index 0 get",
   "index 6 get",
   "index 2 get"
   /* nothing for 3/4 */];

// Do this before the assertions below futz even more with |observed|.
assert.sameValue(observed.length, expectedObserved.length);
for (var i = 0; i < expectedObserved.length; i++)
  assert.sameValue(observed[i], expectedObserved[i]);

assert.sameValue(arr[0], "index 0 get"); // no deletion, setting doesn't overwrite
assert.sameValue(arr[1], "index 6 get"); // copies result of getter
assert.sameValue(2 in arr, false); // deleted
assert.sameValue(3 in arr, false); // never there
assert.sameValue(4 in arr, false); // never there
assert.sameValue(arr[5], "index 2 get"); // copies result of getter
assert.sameValue(6 in arr, false); // deleted
assert.sameValue(arr[7], "index 7 get"); // no deletion, setter doesn't overwrite
