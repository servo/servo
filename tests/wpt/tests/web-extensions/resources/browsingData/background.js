browser.test.runTests([
  /**
   * Tests `browser.browsingData.settings` object properties.
   */
  async function testBrowsingDataSettings() {
    const result = await browser.browsingData.settings();
    browser.test.assertEq(typeof result, 'object',
                          'settings() should return an object');
    browser.test.assertEq(typeof result.options, 'object',
                          'result.options should be an object');
    browser.test.assertEq(typeof result.options.since, 'number',
                          'options.since should be a number');
    browser.test.assertEq(typeof result.dataRemovalPermitted, 'object',
                          'result.dataRemovalPermitted should be an object');
    browser.test.assertEq(typeof result.dataRemovalPermitted.cache, 'boolean',
                          'dataRemovalPermitted.cache should be a boolean');
    browser.test.assertEq(typeof result.dataRemovalPermitted.cookies, 'boolean',
                          'dataRemovalPermitted.cookies should be a boolean');
  },

  /**
   * Tests `browser.browsingData.remove`.
   */
  async function testBrowsingDataRemove() {
    await browser.browsingData.remove({since: 0}, {cache: true, cookies: true});
  },

  /**
   * Tests `browser.browsingData.removeCache`.
   */
  async function testBrowsingDataRemoveCache() {
    await browser.browsingData.removeCache({since: 0});
  },

  /**
   * Tests `browser.browsingData.removeCookies`.
   */
  async function testBrowsingDataRemoveCookies() {
    await browser.browsingData.removeCookies({since: 0});
  },

  /**
   * Tests `browser.browsingData.removeDownloads`.
   */
  async function testBrowsingDataRemoveDownloads() {
    await browser.browsingData.removeDownloads({since: 0});
  },

  /**
   * Tests `browser.browsingData.removeHistory`.
   */
  async function testBrowsingDataRemoveHistory() {
    await browser.browsingData.removeHistory({since: 0});
  },

  /**
   * Tests `browser.browsingData.removeLocalStorage`.
   */
  async function testBrowsingDataRemoveLocalStorage() {
    await browser.browsingData.removeLocalStorage({since: 0});
  },

  /**
   * Tests `browser.browsingData` error cases across multiple methods.
   */
  function testBrowsingDataErrorCases() {
    // `remove` throws on invalid options argument type.
    browser.test.assertThrows(
        () => browser.browsingData.remove('invalid', {cache: true}));

    // `remove` throws on invalid dataTypes argument type.
    browser.test.assertThrows(
        () => browser.browsingData.remove({since: 0}, 'invalid'));

    // `removeCache` throws on invalid options argument type.
    browser.test.assertThrows(() =>
                                  browser.browsingData.removeCache('invalid'));

    // `removeCookies` throws on invalid options argument type.
    browser.test.assertThrows(
        () => browser.browsingData.removeCookies('invalid'));

    // `removeHistory` throws on invalid options argument type.
    browser.test.assertThrows(
        () => browser.browsingData.removeHistory('invalid'));
  }
]);
