browser.test.runTests([
    function browserRuntimeGetURLErrorCases() {
        browser.test.assertThrows(() => browser.runtime.getURL())
        browser.test.assertThrows(() => browser.runtime.getURL(true))
        browser.test.assertThrows(() => browser.runtime.getURL(null))
        browser.test.assertThrows(() => browser.runtime.getURL(undefined))
        browser.test.assertThrows(() => browser.runtime.getURL(42))
        browser.test.assertThrows(() => browser.runtime.getURL(/test/))
        browser.test.assertThrows(() => browser.runtime.getURL({}))
        browser.test.assertThrows(() => browser.runtime.getURL(["test.js"]))
    },
    function browserRuntimeGetURLNormalCases() {
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

        browser.test.assertEq(typeof platformInfo, "object")
        browser.test.assertEq(typeof platformInfo.os, "string")
        browser.test.assertEq(typeof platformInfo.arch, "string")
    },
    async function browserRuntimeGetVersion() {
        const version = browser.runtime.getVersion()
        browser.test.assertEq(typeof version, "string")
        // Implementations are free on how they interpret the version number.
        // However, they need to be consistent in APIs (like the management API).
        browser.test.assertTrue(version === "1.01" || version === "1.1")
        if (browser.management && browser.management.getSelf) {
            const extensionInfo = await browser.management.getSelf()
            browser.test.assertEq(version, extensionInfo.version)
        }
    }
])
