/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Better/more correct handling for replacer arrays with getter array index properties
info: bugzilla.mozilla.org/show_bug.cgi?id=648471
esid: pending
---*/

var replacer = [0, 1, 2, 3];
Object.prototype[3] = 3;
Object.defineProperty(replacer, 1, {
  get: function()
  {
    Object.defineProperty(replacer, 4, { value: 4 });
    delete replacer[2];
    delete replacer[3];
    replacer[5] = 5;
    return 1;
  }
});

var s =
  JSON.stringify({0: { 1: { 3: { 4: { 5: { 2: "omitted" } } } } } }, replacer);

// The replacer array's length is as seen on first query, so property names are
// accumulated for indexes i ∈ {0, 1, 2, 3}, but index 1 deletes 2 and 3, so 2
// isn't seen but 3 is seen as Object.prototype[3].
assert.sameValue('{"0":{"1":{"3":{"3":3}},"3":3},"3":3}', s);


var replacer = [0, 1, 2, 3];
Object.defineProperty(replacer, 0, {
  get: function()
  {
    replacer.length = 0;
    return {};
  }
});

// The replacer.length truncation means only properties on the prototype chain
// shine through, but it doesn't affect the original bounds of the iteration
// used to determine property names which will be included in the final string.
assert.sameValue(JSON.stringify({ 0: 0, 1: 1, 2: 2, 3: 3 }, replacer),
         '{"3":3}');
