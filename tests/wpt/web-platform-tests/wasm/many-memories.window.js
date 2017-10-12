// Copyright 2017 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This test makes sure browsers behave reasonably when asked to allocate a
// large number of WebAssembly.Memory objects at once.
test(function() {
  let memories = [];
  try {
    for (let i = 0; i < 600; i++) {
      memories.push(new WebAssembly.Memory({initial: 1}));
    }
  } catch (e) {
    if (e instanceof RangeError) {
      return;
    }
    throw e;
  }
}, "WebAssembly#CreateManyMemories");
