// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

class VerifyGetUndefinedURLSelectionOperation {
  async run(urls, data) {
    if (await sharedStorage.get('key-not-previously-set') === undefined) {
      return 1;
    }

    return -1;
  }
}

register(
    'verify-get-undefined-url-selection-operation',
    VerifyGetUndefinedURLSelectionOperation);
