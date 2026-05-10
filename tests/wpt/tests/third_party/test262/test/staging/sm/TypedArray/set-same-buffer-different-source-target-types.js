// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  When setting a typed array from an overlapping typed array of different element type, copy the source elements into properly-sized temporary memory, and properly copy them into the target without overflow (of either source *or* target) when finished
info: bugzilla.mozilla.org/show_bug.cgi?id=896116
esid: pending
---*/

/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/
 */

// smallest 2**n triggering segfaults in a pre-patch build locally, then
// quadrupled in case the boundary ever changes, or is different in some other
// memory-allocating implementation
var srclen = 65536;

var ta = new Uint8Array(srclen * Float64Array.BYTES_PER_ELEMENT);
var ta2 = new Float64Array(ta.buffer, 0, srclen);
ta.set(ta2);

// This test mostly exists to check for no crash above, but it's worth testing
// for no uninitialized memory (in case of buffer overflow) being copied into
// the array, too.
for (var i = 0, len = ta.length; i < len; i++)
  assert.sameValue(ta[i], 0, "zero-bits double should convert to zero");
