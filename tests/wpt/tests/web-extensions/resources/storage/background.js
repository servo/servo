const area = browser.storage.local;
const key1 = 'key1';
const val1 = 'val1';
const key2 = 'key2';
const val2 = 'val2';

browser.test.runTests([
  /**
   * Tests `StorageArea.onChanged` event.
   */
  async function testStorageOnChanged() {
    await area.clear();
    const changePromise = new Promise((resolve) => {
      area.onChanged.addListener(function listener(changes) {
        area.onChanged.removeListener(listener);
        resolve(changes);
      });
    });

    // Trigger change by setting `key1`.
    await area.set({[key1]: val1});

    const changes = await changePromise;
    browser.test.assertTrue(key1 in changes,
                            `Changes should contain '${key1}'`);
    browser.test.assertEq(changes[key1].newValue, val1,
                          'New value should match');
    browser.test.assertEq(changes[key1].oldValue, undefined,
                          'Old value should be undefined');
    await area.clear();
  },

  /**
   * Tests `StorageArea.getBytesInUse`.
   */
  async function testStorageGetBytesInUse() {
    await area.clear();
    await area.set({[key1]: val1});
    const bytesInUse = await area.getBytesInUse(key1);
    browser.test.assertEq(typeof bytesInUse, 'number',
                          'Bytes in use should be a number');
    browser.test.assertTrue(bytesInUse > 0,
                            'Bytes in use should be greater than 0');
    await area.clear();
  },

  /**
   * Tests `StorageArea.set` and `StorageArea.getKeys`.
   */
  async function testStorageSetAndGetKeys() {
    await area.clear();
    await area.set({[key1]: val1, [key2]: val2});
    const keys = await area.getKeys();
    // Order is not guaranteed, so check inclusion.
    browser.test.assertTrue(keys.includes(key1),
                            `Keys should include '${key1}'`);
    browser.test.assertTrue(keys.includes(key2),
                            `Keys should include '${key2}'`);
    await area.clear();
  },

  /**
   * Tests `StorageArea.get` with `null` (get all).
   */
  async function testStorageGetAll() {
    await area.clear();
    await area.set({[key1]: val1, [key2]: val2});
    const allItems = await area.get(null);
    browser.test.assertEq(allItems[key1], val1,
                          'All items should contain key1');
    browser.test.assertEq(allItems[key2], val2,
                          'All items should contain key2');
    await area.clear();
  },

  /**
   * Tests `StorageArea.get` with default values.
   */
  async function testStorageGetWithDefault() {
    await area.clear();
    const defaultKey = 'defaultKey';
    const defaultValue = 'defaultValue';
    const getWithDefault = await area.get({[defaultKey]: defaultValue});
    browser.test.assertEq(getWithDefault[defaultKey], defaultValue,
                          'Should return default value if key not present');
    await area.clear();
  },

  /**
   * Tests `StorageArea.remove`.
   */
  async function testStorageRemove() {
    await area.clear();
    await area.set({[key1]: val1, [key2]: val2});
    await area.remove(key1);
    const keysAfterRemove = await area.getKeys();
    browser.test.assertFalse(keysAfterRemove.includes(key1),
                             `Keys should not include '${key1}' after remove`);
    browser.test.assertTrue(keysAfterRemove.includes(key2),
                            `Keys should still include '${key2}'`);
    await area.clear();
  },

  /**
   * Tests `StorageArea.setAccessLevel` with TRUSTED_CONTEXTS access level.
   */
  async function testStorageSetAccessLevelTrustedContexts() {
    await area.setAccessLevel({accessLevel: 'TRUSTED_CONTEXTS'});
  },

  /**
   * Tests `StorageArea.setAccessLevel` with TRUSTED_AND_UNTRUSTED_CONTEXTS
   * access level.
   */
  async function testStorageSetAccessLevelTrustedAndUntrustedContexts() {
    await area.setAccessLevel({accessLevel: 'TRUSTED_AND_UNTRUSTED_CONTEXTS'});
  },

  /**
   * Tests `StorageArea.clear`.
   */
  async function testStorageClear() {
    await area.clear();
    await area.set({[key1]: val1});
    const itemsBeforeClear = await area.get(key1);
    browser.test.assertEq(itemsBeforeClear[key1], val1,
                          'Value should be set prior to clear');
    await area.clear();
    const itemsAfterClear = await area.get(key1);
    browser.test.assertEq(itemsAfterClear[key1], undefined,
                          'Value should be undefined after clear');
    const finalKeys = await area.getKeys();
    browser.test.assertEq(finalKeys.length, 0,
                          'Storage should be empty after clear');
  },

  /**
   * Tests `browser.storage.local` error cases.
   */
  function testStorageErrorCases() {
    // Verify `set` throws on invalid arguments (requires `object`).
    browser.test.assertThrows(() => area.set('invalid'));
    browser.test.assertThrows(() => area.set(null));

    // Verify `remove` throws on invalid arguments (requires `string` or `array`
    // of `string`s).
    browser.test.assertThrows(() => area.remove(null));
    browser.test.assertThrows(() => area.remove(123));

    // Verify `setAccessLevel` throws on invalid arguments.
    browser.test.assertThrows(() => area.setAccessLevel('invalid'));
    browser.test.assertThrows(
        () => area.setAccessLevel({accessLevel: 'INVALID'}));
  }
]);
