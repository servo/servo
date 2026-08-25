/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Assertion redefining non-writable length to a non-numeric value
info: bugzilla.mozilla.org/show_bug.cgi?id=866700
esid: pending
---*/

var arr = [];
Object.defineProperty(arr, "length", { value: 0, writable: false });

// Per Array's magical behavior, the value in the descriptor gets canonicalized
// *before* SameValue comparisons occur, so this shouldn't throw.
Object.defineProperty(arr, "length", { value: '' });

assert.sameValue(arr.length, 0);
