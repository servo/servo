/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
features: [host-gc-required]
---*/

var obj = {a: 0, b: 1, c: 2};
delete obj.b;  // switch to dictionary mode
Object.defineProperty(obj, 'g',
                      {get: function () { return -1; }, configurable: true});
for (var i = 3; i < 20; i++)
    obj['x' + i] = i;  // get property table
for (var i = 3; i < 20; i++)
    delete obj['x' + i];  // add to freelist
delete obj.g;  // must update lastProp->freeslot, to avoid assertion

// extra junk to try to hit the assertion, if freeslot is not updated
$262.gc();
obj.d = 3;
obj.e = 4;

