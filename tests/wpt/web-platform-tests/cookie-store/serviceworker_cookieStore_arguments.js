self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

importScripts(
    "cookieStore_delete_arguments.tentative.window.js",
    "cookieStore_get_arguments.tentative.window.js",
    "cookieStore_getAll_arguments.tentative.window.js",
    "cookieStore_set_arguments.tentative.window.js");

done();
