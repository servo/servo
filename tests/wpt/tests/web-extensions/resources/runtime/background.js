browser.test.runTests([
  function browserRuntimeGetURLErrorCases() {
    browser.test.assertThrows(() => browser.runtime.getURL())
    browser.test.assertThrows(() => browser.runtime.getURL(true))
    browser.test.assertThrows(() => browser.runtime.getURL(null))
    browser.test.assertThrows(() => browser.runtime.getURL(undefined))
    browser.test.assertThrows(() => browser.runtime.getURL(42))
    browser.test.assertThrows(() => browser.runtime.getURL(/test/))
    browser.test.assertThrows(() => browser.runtime.getURL({}))
    browser.test.assertThrows(() => browser.runtime.getURL(['test.js']))
  },
  function
      browserRuntimeGetURLNormalCases() {
        browser.test.assertEq(typeof browser.runtime.getURL(""), "string")
        browser.test.assertEq(new URL(browser.runtime.getURL("")).pathname, "/")
        browser.test.assertEq(new URL(browser.runtime.getURL("test.js")).pathname, "/test.js")
        browser.test.assertEq(new URL(browser.runtime.getURL("/test.js")).pathname, "/test.js")
        browser.test.assertEq(new URL(browser.runtime.getURL("../../test.js")).pathname, "/test.js")
        browser.test.assertEq(new URL(browser.runtime.getURL("./test.js")).pathname, "/test.js")
        browser.test.assertEq(new URL(browser.runtime.getURL("././/example")).pathname, "//example")
        browser.test.assertEq(new URL(browser.runtime.getURL("../../example/..//test/")).pathname, "//test/")
        browser.test.assertEq(new URL(browser.runtime.getURL(".")).pathname, "/")
        browser.test.assertEq(new URL(browser.runtime.getURL("..//../")).pathname, "/")
        browser.test.assertEq(new URL(browser.runtime.getURL(".././..")).pathname, "/")
        browser.test.assertEq(new URL(browser.runtime.getURL("/.././.")).pathname, "/")
      },
  async function browserRuntimeGetPlatformInfo() {
    const platformInfo = await browser.runtime.getPlatformInfo()

    browser.test.assertEq(typeof platformInfo, 'object')
    browser.test.assertEq(typeof platformInfo.os, 'string')
    browser.test.assertEq(typeof platformInfo.arch, 'string')
  },
  async function browserRuntimeGetVersion() {
    const version = browser.runtime.getVersion()
    browser.test.assertEq(typeof version, 'string')
    // Implementations are free on how they interpret the version number.
    // However, they need to be consistent in APIs (like the management API).
    browser.test.assertTrue(version === '1.01' || version === '1.1')
    if (browser.management && browser.management.getSelf) {
      const extensionInfo = await browser.management.getSelf()
      browser.test.assertEq(version, extensionInfo.version)
    }
  },
  /**
   * Tests `browser.runtime.onEnabled` event existence, listener
   * registration, query, and removal.
   */
  function
      testBrowserRuntimeOnEnabled() {
        // Verify that `browser.runtime.onEnabled` exists and is an object.
        browser.test.assertEq(
            typeof browser.runtime.onEnabled, 'object',
            '`browser.runtime.onEnabled` should be an object');

        // Verify initial listener state.
        const listener = () => {};
        browser.test.assertFalse(
            browser.runtime.onEnabled.hasListener(listener),
            '`hasListener` should return false before listener registration');

        // Register a listener on `browser.runtime.onEnabled`.
        browser.runtime.onEnabled.addListener(listener);
        browser.test.assertTrue(
            browser.runtime.onEnabled.hasListener(listener),
            '`hasListener` should return true after listener registration');
        browser.test.assertTrue(
            browser.runtime.onEnabled.hasListeners(),
            '`hasListeners` should return true when listeners are registered');

        // Remove the listener and verify updated listener state.
        browser.runtime.onEnabled.removeListener(listener);
        browser.test.assertFalse(
            browser.runtime.onEnabled.hasListener(listener),
            '`hasListener` should return false after listener removal');
      },
  /**
   * Tests `browser.runtime.onExtensionLoaded` event and
   * `browser.runtime.OnLoadedReason` enum values.
   */
  function testBrowserRuntimeOnExtensionLoaded() {
    // Verify that `browser.runtime.onExtensionLoaded` exists and is an
    // object.
    browser.test.assertEq(
        typeof browser.runtime.onExtensionLoaded, 'object',
        '`browser.runtime.onExtensionLoaded` should be an object');

    // Verify initial listener state.
    const listener = (details) => {};
    browser.test.assertFalse(
        browser.runtime.onExtensionLoaded.hasListener(listener),
        '`hasListener` should return false before listener registration');

    // Register a listener on `browser.runtime.onExtensionLoaded`.
    browser.runtime.onExtensionLoaded.addListener(listener);
    browser.test.assertTrue(
        browser.runtime.onExtensionLoaded.hasListener(listener),
        '`hasListener` should return true after listener registration');
    browser.test.assertTrue(
        browser.runtime.onExtensionLoaded.hasListeners(),
        '`hasListeners` should return true when listeners are registered');

    // Remove the listener and verify updated listener state.
    browser.runtime.onExtensionLoaded.removeListener(listener);
    browser.test.assertFalse(
        browser.runtime.onExtensionLoaded.hasListener(listener),
        '`hasListener` should return false after listener removal');

    // Verify `browser.runtime.OnLoadedReason` enum values match the
    // specification.
    browser.test.assertEq(
        typeof browser.runtime.OnLoadedReason, 'object',
        '`browser.runtime.OnLoadedReason` should be an object');
    const enumKeys = Object.keys(browser.runtime.OnLoadedReason);
    browser.test.assertEq(
        enumKeys.length, 6,
        '`browser.runtime.OnLoadedReason` should have exactly 6 enum entries');
    browser.test.assertEq(browser.runtime.OnLoadedReason.INSTALL, 'install',
                          '`OnLoadedReason.INSTALL` should match "install"');
    browser.test.assertEq(browser.runtime.OnLoadedReason.UPDATE, 'update',
                          '`OnLoadedReason.UPDATE` should match "update"');
    browser.test.assertEq(
        browser.runtime.OnLoadedReason.BROWSER_UPDATE, 'browser_update',
        '`OnLoadedReason.BROWSER_UPDATE` should match "browser_update"');
    browser.test.assertEq(browser.runtime.OnLoadedReason.ENABLE, 'enable',
                          '`OnLoadedReason.ENABLE` should match "enable"');
    browser.test.assertEq(browser.runtime.OnLoadedReason.STARTUP, 'startup',
                          '`OnLoadedReason.STARTUP` should match "startup"');
    browser.test.assertEq(browser.runtime.OnLoadedReason.RELOAD, 'reload',
                          '`OnLoadedReason.RELOAD` should match "reload"');
  }
])
