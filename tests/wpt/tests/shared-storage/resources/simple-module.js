// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

var globalVar = 0;

class TestURLSelectionOperation {
  async run(urls, data) {
    if (data && data.hasOwnProperty('setKey') && data.hasOwnProperty('setValue')) {
      await sharedStorage.set(data['setKey'], data['setValue']);
    }

    if (data && data.hasOwnProperty('mockResult')) {
      return data['mockResult'];
    }

    return -1;
  }
}

class IncrementGlobalVariableAndReturnOriginalValueOperation {
  async run(urls, data) {
    return globalVar++;
  }
}

class VerifyKeyValue {
  async run(urls, data) {
    if (data && data.hasOwnProperty('expectedKey') &&
        data.hasOwnProperty('expectedValue')) {
      const expectedValue = data['expectedValue'];
      const value = await sharedStorage.get(data['expectedKey']);
      if (value === expectedValue) {
        return 1;
      }
    }
    return -1;
  }
}

class VerifyKeyNotFound {
  async run(urls, data) {
    if (data && data.hasOwnProperty('expectedKey')) {
      const value = await sharedStorage.get(data['expectedKey']);
      if (typeof value === 'undefined') {
        return 1;
      }
    }
    return -1;
  }
}

register('test-url-selection-operation', TestURLSelectionOperation);
register('increment-global-variable-and-return-original-value-operation',
         IncrementGlobalVariableAndReturnOriginalValueOperation);
register('verify-key-value', VerifyKeyValue);
register('verify-key-not-found', VerifyKeyNotFound);
