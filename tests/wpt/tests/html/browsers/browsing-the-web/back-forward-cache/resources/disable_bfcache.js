// Try to disable BFCache by using the Keyboard Lock API.
// Note: This is a workaround, and we might want to introduce a test-only
// BFCache-disabling API instead in the future.
// https://github.com/web-platform-tests/wpt/issues/16359#issuecomment-795004780
// https://crbug.com/1298336
window.disableBFCache = () => {
  return new Promise(resolve => {
    navigator.keyboard.lock();
    resolve();
  });
};
