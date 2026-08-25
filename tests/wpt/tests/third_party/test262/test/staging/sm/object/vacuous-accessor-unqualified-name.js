/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Using a name referring to a { get: undefined, set: undefined } descriptor shouldn't assert
info: bugzilla.mozilla.org/show_bug.cgi?id=560216
esid: pending
---*/

Object.defineProperty(this, "x", { set: undefined, configurable: true });
x;
