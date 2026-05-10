// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Freezing a dictionary mode object with a length property should make Object.isFrozen report true
info: bugzilla.mozilla.org/show_bug.cgi?id=1312948
esid: pending
---*/
/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 * Author: Emilio Cobos Álvarez <ecoal95@gmail.com>
 */

/* Convert to dictionary mode */
delete Array.prototype.slice;

Object.freeze(Array.prototype);
assert.sameValue(Object.isFrozen(Array.prototype), true);
