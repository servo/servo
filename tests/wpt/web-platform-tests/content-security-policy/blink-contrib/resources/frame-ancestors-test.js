var SAME_ORIGIN = true;
var CROSS_ORIGIN = false;

var EXPECT_BLOCK = true;
var EXPECT_LOAD = false;

var SAMEORIGIN_ORIGIN = "http://127.0.0.1:8000";
var CROSSORIGIN_ORIGIN = "http://localhost:8080";

window.jsTestIsAsync = true;
window.wasPostTestScriptParsed = true;

if (window.testRunner)
    testRunner.dumpChildFramesAsText();

window.addEventListener("message", function(e) {
    if (window.parent != window) {
        window.parent.postMessage(e.data, "*");
    } else {
        if (e.data)
            testFailed("The inner IFrame failed.");
        else
            testPassed("The inner IFrame passed.");

        finishJSTest();
    }
});

function injectNestedIframe(policy, parent, child, expectation) {
    var iframe = document.createElement("iframe");

    var url = "/security/contentSecurityPolicy/resources/frame-in-frame.pl?" + "policy=" + policy + "&parent=" + parent + "&child=" + child + "&expectation=" + expectation;
    url = (parent == "same" ? SAMEORIGIN_ORIGIN : CROSSORIGIN_ORIGIN) + url;

    iframe.src = url;
    document.body.appendChild(iframe);
}

function injectIFrame(policy, sameOrigin, expectBlock) {
    var iframe = document.createElement("iframe");
    iframe.addEventListener("load", iframeLoaded(expectBlock));
    iframe.addEventListener("error", iframeLoaded(expectBlock));

    var url = "/security/contentSecurityPolicy/resources/frame-ancestors.pl?policy=" + policy;
    if (!sameOrigin)
        url = CROSSORIGIN_ORIGIN + url;

    iframe.src = url;
    document.body.appendChild(iframe);
}

function iframeLoaded(expectBlock) {
    return function(ev) {
        var failed = true;
        try {
            console.log("IFrame load event fired: the IFrame's location is '" + ev.target.contentWindow.location.href + "'.");
            if (expectBlock) {
                testFailed("The IFrame should have been blocked (or cross-origin). It wasn't.");
                failed = true;
            } else {
                testPassed("The IFrame should not have been blocked. It wasn't.");
                failed = false;
            }
        } catch (ex) {
            debug("IFrame load event fired: the IFrame is cross-origin (or was blocked).");
            if (expectBlock) {
                testPassed("The IFrame should have been blocked (or cross-origin). It was.");
                failed = false;
            } else {
                testFailed("The IFrame should not have been blocked. It was.");
                failed = true;
            }
        }
        if (window.parent != window)
            window.parent.postMessage(failed, '*');
        else
            finishJSTest();
    };
}

function crossOriginFrameShouldBeBlocked(policy) {
    window.onload = function() {
        injectIFrame(policy, CROSS_ORIGIN, EXPECT_BLOCK);
    };
}

function crossOriginFrameShouldBeAllowed(policy) {
    window.onload = function() {
        injectIFrame(policy, CROSS_ORIGIN, EXPECT_LOAD);
    };
}

function sameOriginFrameShouldBeBlocked(policy) {
    window.onload = function() {
        injectIFrame(policy, SAME_ORIGIN, EXPECT_BLOCK);
    };
}

function sameOriginFrameShouldBeAllowed(policy) {
    window.onload = function() {
        injectIFrame(policy, SAME_ORIGIN, EXPECT_LOAD);
    };
}

function testNestedIFrame(policy, parent, child, expectation) {
    window.onload = function() {
        injectNestedIframe(policy, parent == SAME_ORIGIN ? "same" : "cross", child == SAME_ORIGIN ? "same" : "cross", expectation == EXPECT_LOAD ? "Allowed" : "Blocked");
    };
}
