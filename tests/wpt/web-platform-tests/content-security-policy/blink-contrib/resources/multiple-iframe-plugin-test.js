if (window.testRunner) {
    testRunner.waitUntilDone();
    testRunner.dumpAsText();
    testRunner.dumpChildFramesAsText();
}

function testExperimentalPolicy() {
    testImpl(true);
}

function test() {
    testImpl(false);
}

function testImpl(experimental) {
    if (tests.length === 0)
        return finishTesting();
    var baseURL = "/security/contentSecurityPolicy/";
    var current = tests.shift();
    var iframe = document.createElement("iframe");
    iframe.src = baseURL + "resources/echo-object-data.pl?" +
        "experimental=" + (experimental ? "true" : "false") +
        "&csp=" + escape(current[1]);

    if (current[0])
        iframe.src += "&log=PASS.";
    else
        iframe.src += "&log=FAIL.";

    if (current[2])
        iframe.src += "&plugin=" + escape(current[2]);
    else {
        iframe.src += "&plugin=data:application/x-webkit-test-netscape,logifloaded";
    }

    if (current[3] !== undefined)
        iframe.src += "&type=" + escape(current[3]);
    else
        iframe.src += "&type=application/x-webkit-test-netscape";

    iframe.onload = function() {
        if (window.internals)
            internals.updateLayoutIgnorePendingStylesheetsAndRunPostLayoutTasks(iframe);
        testImpl(experimental);
    };
    document.body.appendChild(iframe);
}

function finishTesting() {
    if (window.testRunner) {
        setTimeout("testRunner.notifyDone()", 0);
    }
    return true;
}
