<<<<<<< HEAD
// Todo: cache Navigator object
//is(window.navigator, window.navigator);
is(String(window.navigator), '[object Navigator]');
=======
// Sanity checks
is(window.navigator, window.navigator);
// todo should be [object Navigator]
is(String(window.navigator), 'Navigator');
>>>>>>> 416352bb2e0c6eb7fe9280d22d0cb6a03f195e6e

var nav = window.navigator;
is(nav.doNotTrack, "unspecified");
is(nav.vendor, "");
is(nav.vendorSub, "");
is(nav.product, "Gecko");
is(nav.javaEnabled(), false);
is(nav.taintEnabled(), false);
is(nav.appName, "Netscape");
is(nav.appCodeName, "Mozilla");
// todo
is(nav.appVersion, null);
is(nav.platform, null);
is(nav.userAgent, null);
is(nav.language, null);
is(nav.onLine, true);
