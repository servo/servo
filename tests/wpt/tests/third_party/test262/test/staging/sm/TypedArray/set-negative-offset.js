// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  %TypedArray%.prototype.set must throw a RangeError when passed a negative offset
info: bugzilla.mozilla.org/show_bug.cgi?id=1140752
esid: pending
---*/

/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/
 */

assert.throws(RangeError, function() {
  new Uint8Array().set([], -1);
});
