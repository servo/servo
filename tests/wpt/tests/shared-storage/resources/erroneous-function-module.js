// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

class TestURLSelectionOperation {
  async run(urls, data) {
    await sharedStorage.append('run-attempt', '1');
    undefinedVariable;
    return -1;
  }
}

class VerifyRunAttempts {
  async run(urls, data) {
    const attempts = await sharedStorage.get('run-attempt');
    if (!attempts) {
      return -1;
    }
    return attempts.length;
  }
}

register('test-url-selection-operation', TestURLSelectionOperation);
register('verify-run-attempts', VerifyRunAttempts);
