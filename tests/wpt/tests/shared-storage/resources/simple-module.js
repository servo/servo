// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

var globalVar = 0;

async function busyWaitMs(time_to_wait) {
  const startTime = Date.now();
  while (Date.now() - startTime < time_to_wait) {

  }
}

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

class TestURLSelectionOperationTwo {
  async run(urls, data) {
    if (data && data.hasOwnProperty('mockResult')) {
      return data['mockResult'];
    }

    return -1;
  }
}

class TestSlowURLSelectionOperation {
  async run(urls, data) {
    await busyWaitMs(100);
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

class VerifyInterestGroups {
  async run(urls, data) {
    if (data &&
        data.hasOwnProperty('expectedOwner') &&
        data.hasOwnProperty('expectedName')) {

      const groups = await interestGroups();

      if (groups.length !== 1) {
        return -1;
      }

      if (groups[0]["owner"] !== data['expectedOwner']) {
        return -1;
      }

      if (groups[0]["name"] !== data['expectedName']) {
        return -1;
      }

      return 1;
    }
    return -1;
  }
}

class GetWaitIncrementWithinLockOperation {
  async run(urls, data) {
    if (data && data.hasOwnProperty('key')) {
      await navigator.locks.request("lock0", async (lock) => {
        let value_read = await sharedStorage.get(data['key']);
        value_read = value_read ? Number(value_read) : 0;

        await busyWaitMs(100);

        await sharedStorage.set(data['key'], value_read + 1);
      });

      return 1;
    }
    return -1;
  }
}

register('test-url-selection-operation', TestURLSelectionOperation);
register('test-url-selection-operation-2', TestURLSelectionOperationTwo);
register('test-slow-url-selection-operation', TestSlowURLSelectionOperation);
register('increment-global-variable-and-return-original-value-operation',
         IncrementGlobalVariableAndReturnOriginalValueOperation);
register('verify-key-value', VerifyKeyValue);
register('verify-key-not-found', VerifyKeyNotFound);
register('verify-interest-groups', VerifyInterestGroups);
register('get-wait-increment-within-lock', GetWaitIncrementWithinLockOperation);
