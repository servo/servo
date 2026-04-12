// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Force a sleep for 0.5 second, to make sure the rest of the parsing is done
// via a timer.
(function () {
  const start = new Date().getTime();
  while (true) {
    const now = new Date().getTime();
    if (now - start > 500) {
      break;
    }
  }
})();
