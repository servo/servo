if (window.testRunner) {
    testRunner.dumpAsText();
    testRunner.waitUntilDone();
}

window.onload = function() {
    var test = window.location.pathname.replace(/^.+\//, '');
    var match = window.location.search.match(/^\?test=([^&]+)/);
    if (match)
        test = match[1];
    window.location = "/security/contentSecurityPolicy/resources/echo-report.php?test=" + test;
}
