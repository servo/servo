if (window.testRunner) {
    testRunner.waitUntilDone();
    testRunner.dumpAsText();
    testRunner.dumpChildFramesAsText();
}

function testPreescapedPolicy() {
    testImpl(false, true);
}

function testExperimentalPolicy() {
    testImpl(true, false);
}

function test() {
    testImpl(false, false);
}

function testImpl(experimental, preescapedPolicy) {
    if (tests.length === 0)
        return finishTesting();

    var baseURL = "/security/contentSecurityPolicy/";
    var current = tests.shift();
    var iframe = document.createElement("iframe");

    var policy = current[1];
    if (!preescapedPolicy)
        policy = encodeURIComponent(policy);

    var scriptToLoad = baseURL + encodeURIComponent(current[2]);
    if (current[2].match(/^data:/) || current[2].match(/^https?:/))
        scriptToLoad = encodeURIComponent(current[2]);

    iframe.src = baseURL + "resources/echo-script-src.pl?" +
        "experimental=" + (experimental ? "true" : "false") +
        "&should_run=" + encodeURIComponent(current[0]) +
        "&csp=" + policy + "&q=" + scriptToLoad;
    if (current[3] !== undefined)
        iframe.src += "&nonce=" + encodeURIComponent(current[3]);

    iframe.onload = function() {
        testImpl(experimental, preescapedPolicy);
    };
    document.body.appendChild(iframe);
}

function finishTesting() {
    if (window.testRunner) {
        setTimeout("testRunner.notifyDone()", 0);
    }
    return true;
}
