/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  JSON.stringify shouldn't use context-wide cycle detection
info: bugzilla.mozilla.org/show_bug.cgi?id=1197097
esid: pending
---*/

var arr;

// Nested yet separate JSON.stringify is okay.
arr = [{}];
assert.sameValue(JSON.stringify(arr, function(k, v) {
  assert.sameValue(JSON.stringify(arr), "[{}]");
  return v;
}), "[{}]");

// SpiderMonkey censors cycles in array-joining.  This mechanism must not
// interfere with the cycle detection in JSON.stringify.
arr = [{
  toString: function() {
    var s = JSON.stringify(arr);
    assert.sameValue(s, "[{}]");
    return s;
  }
}];
assert.sameValue(arr.join(), "[{}]");
