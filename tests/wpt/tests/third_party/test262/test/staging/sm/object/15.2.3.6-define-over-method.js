/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Do not assert: !(attrs & (JSPROP_GETTER | JSPROP_SETTER)) with Object.defineProperty
info: bugzilla.mozilla.org/show_bug.cgi?id=568786
esid: pending
---*/

var o = { x: function(){} };
Object.defineProperty(o, "x", { get: function(){} });
