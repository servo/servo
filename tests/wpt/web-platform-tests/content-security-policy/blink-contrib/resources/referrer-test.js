var ReferrerTest = {
    NO_REFERRER: "no-referrer",
    NO_REFERRER_WHEN_DOWNGRADE: "no-referrer-when-downgrade",
    ORIGIN: "origin",
    ORIGIN_WHEN_CROSS_ORIGIN: "origin-when-cross-origin",
    UNSAFE_URL: "unsafe-url",

    INVALID: "invalid",
    EMPTY: "",

    HTTP: "http",
    HTTPS: "https",

    bindHandler: function(func) {
        window.addEventListener("message", function(e) {
            ReferrerTest.referrerResult = undefined;
            func(e.data);
            finishJSTest();
        });
    },

    base: function(scheme) {
        return scheme == "http" ? "http://127.0.0.1:8000/" : "https://127.0.0.1:8443/";
    },

    generateFrameURL: function(policy, from, to) {
        return ReferrerTest.base(from) + "security/contentSecurityPolicy/resources/referrer-test.php?policy=" + policy + "&to=" + to;
    },

    injectFrame: function(policy, from, to) {
        var iframe = document.createElement("iframe");
        iframe.src = ReferrerTest.generateFrameURL(policy, from, to);
        document.body.appendChild(iframe);
    }
};

function expectFullReferrer(policy, from, to) {
    ReferrerTest.bindHandler(function(referrer) {
        ReferrerTest.referrerResult = referrer;
        shouldBeEqualToString("ReferrerTest.referrerResult", ReferrerTest.generateFrameURL(policy, from, to));
    });
    ReferrerTest.injectFrame(policy, from, to);
}

function expectNoReferrer(policy, from, to) {
    ReferrerTest.bindHandler(function(referrer) {
        ReferrerTest.referrerResult = referrer;
        shouldBeEqualToString("ReferrerTest.referrerResult", "");
    });
    ReferrerTest.injectFrame(policy, from, to);
}

function expectOriginReferrer(policy, from, to) {
    ReferrerTest.bindHandler(function(referrer) {
        ReferrerTest.referrerResult = referrer;
        shouldBeEqualToString("ReferrerTest.referrerResult", ReferrerTest.base(from));
    });
    ReferrerTest.injectFrame(policy, from, to);
}

window.wasPostTestScriptParsed = true;
window.jsTestIsAsync = true;
