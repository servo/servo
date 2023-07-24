/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { timeout } from './timeout.js'; // Copied from https://github.com/web-platform-tests/wpt/blob/master/common/reftest-wait.js

/**
 * Remove the `reftest-wait` class on the document element.
 * The reftest runner will wait with taking a screenshot while
 * this class is present.
 *
 * See https://web-platform-tests.org/writing-tests/reftests.html#controlling-when-comparison-occurs
 */
export function takeScreenshot() {
  document.documentElement.classList.remove('reftest-wait');
}

/**
 * Call `takeScreenshot()` after a delay of at least `ms` milliseconds.
 * @param {number} ms - milliseconds
 */
export function takeScreenshotDelayed(ms) {
  timeout(() => {
    takeScreenshot();
  }, ms);
}
