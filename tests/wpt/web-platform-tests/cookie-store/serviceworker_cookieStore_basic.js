self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

importScripts(
    "cookieStore_get_delete_basic.tentative.window.js",
    "cookieStore_get_set_basic.tentative.window.js",
    "cookieStore_getAll_set_basic.tentative.window.js");

done();
