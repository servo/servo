self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

importScripts(
    "cookieListItem_attributes.tentative.https.window.js",
    "cookieStore_delete_arguments.tentative.https.window.js",
    "cookieStore_get_arguments.tentative.https.window.js",
    "cookieStore_getAll_arguments.tentative.https.window.js",
    "cookieStore_getAll_multiple.tentative.https.window.js",
    "cookieStore_set_arguments.tentative.https.window.js",
    "cookieStore_special_names.tentative.https.window.js");

done();
