/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  String.prototype.match should throw when called with a global RegExp whose .lastIndex is non-writable
info: bugzilla.mozilla.org/show_bug.cgi?id=501739
esid: pending
---*/

var s = '0x2x4x6x8';

// First time with .lastIndex === 0

var p1 = /x/g;
Object.defineProperty(p1, "lastIndex", { writable: false });

assert.throws(TypeError, function() {
  s.match(p1);
});

// Second time with .lastIndex !== 0

var p2 = /x/g;
Object.defineProperty(p2, "lastIndex", { writable: false, value: 3 });

assert.throws(TypeError, function() {
  s.match(p2);
});

// Third time with .lastIndex === 0, no matches

var p3 = /q/g;
Object.defineProperty(p3, "lastIndex", { writable: false });

assert.throws(TypeError, function() {
  s.match(p3);
});

// Fourth time with .lastIndex !== 0, no matches

var p4 = /q/g;
Object.defineProperty(p4, "lastIndex", { writable: false, value: 3 });

assert.throws(TypeError, function() {
  s.match(p4);
});
