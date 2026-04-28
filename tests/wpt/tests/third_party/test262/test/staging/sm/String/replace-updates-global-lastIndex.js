/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  String.prototype.relace should zero the .lastIndex when called with a global RegExp
info: bugzilla.mozilla.org/show_bug.cgi?id=501739
esid: pending
---*/

var s = '0x2x4x6x8';

var p1 = /x/g;
p1.lastIndex = 3;
s.replace(p1, '');
assert.sameValue(p1.lastIndex, 0);

var p2 = /x/g;
p2.lastIndex = 3;
var c = 0;
s.replace(p2, function(s) {
    assert.sameValue(p2.lastIndex++, c++);
    return 'y';
});
assert.sameValue(p2.lastIndex, 4);
