self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

importScripts(
    "cookieStore_delete_basic.tentative.https.window.js",
    "cookieStore_get_delete_basic.tentative.https.window.js",
    "cookieStore_get_set_basic.tentative.https.window.js",
    "cookieStore_getAll_set_basic.tentative.https.window.js");

done();
