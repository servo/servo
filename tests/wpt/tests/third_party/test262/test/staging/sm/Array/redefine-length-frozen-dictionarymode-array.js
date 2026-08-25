/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Assertion redefining length property of a frozen dictionary-mode array
info: bugzilla.mozilla.org/show_bug.cgi?id=880591
esid: pending
---*/

function convertToDictionaryMode(arr)
{
  Object.defineProperty(arr, 0, { configurable: true });
  Object.defineProperty(arr, 1, { configurable: true });
  delete arr[0];
}

var arr = [];
convertToDictionaryMode(arr);
Object.freeze(arr);
Object.defineProperty(arr, "length", {});
