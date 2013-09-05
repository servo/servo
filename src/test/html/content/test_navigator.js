is(window.navigator, window.navigator);
is(String(window.navigator), '[object Navigator]');

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
