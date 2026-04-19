/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
info: |
  preventExtensions on global
  bugzilla.mozilla.org/show_bug.cgi?id=600128
description: |
  Properly handle attempted addition of properties to non-extensible objects
esid: pending
---*/

var o = Object.freeze({});
for (var i = 0; i < 10; i++)
  o.u = "";

Object.freeze(this);
for (let j = 0; j < 10; j++)
  u = "";
