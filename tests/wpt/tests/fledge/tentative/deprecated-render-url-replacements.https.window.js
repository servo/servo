// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-last


"use strict;"

// This test ensures proper handling of deprecatedRenderURLReplacements within auctionConfigOverrides.
// It validates that these replacements are correctly applied to the winning bid's renderURL by
// injecting a URL with matching macros into an interest group and ensuring that a new url with
// the replacements in it, is tracked and observed.
const makeTest = ({
    // Test name
    name,
    // Overrides to the interest group.
    interestGroupOverrides = {},
    // Overrides to the auction config.
    auctionConfigOverrides = {},
    // This is what goes into the renderURL and is expected to be replaced.
    beforeReplacements,
    // This is what's expected when 'beforeReplacements' is replaced.
    afterReplacements,
}) => {
    subsetTest(promise_test, async test => {
        const uuid = generateUuid(test);
        let urlBeforeReplacements = createTrackerURL(window.location.origin, uuid, 'track_get', beforeReplacements);
        let urlAfterReplacements = createTrackerURL(window.location.origin, uuid, 'track_get', afterReplacements);
        interestGroupOverrides.ads = [{ renderURL: urlBeforeReplacements }];
        await joinInterestGroup(test, uuid, interestGroupOverrides);

        await runBasicFledgeAuctionAndNavigate(test, uuid, auctionConfigOverrides);
        await waitForObservedRequests(
            uuid,
            [urlAfterReplacements, createSellerReportURL(uuid), createBidderReportURL(uuid)]);
    }, name);
};

makeTest({
    name: 'Replacements with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${EXAMPLE-MACRO}': 'SSP' }
    },
    beforeReplacements: "${EXAMPLE-MACRO}",
    afterReplacements: 'SSP',

});

makeTest({
    name: 'Replacements with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%EXAMPLE-MACRO%%': 'SSP' }
    },
    beforeReplacements: "%%EXAMPLE-MACRO%%",
    afterReplacements: 'SSP',
});

makeTest({
    name: 'Multiple replacements within a URL.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${EXAMPLE-MACRO1}': 'SSP1', '%%EXAMPLE-MACRO2%%': 'SSP2' }
    },
    beforeReplacements: "${EXAMPLE-MACRO1}/%%EXAMPLE-MACRO2%%",
    afterReplacements: 'SSP1/SSP2',
});

makeTest({
    name: 'Recursive and reduce size with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${1}': '1' }
    },
    beforeReplacements: "${${${1}}}",
    afterReplacements: "${${1}}"
});

makeTest({
    name: 'Recursive and increase size with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${1}': '${${1}}' }
    },
    beforeReplacements: "${1}",
    afterReplacements: "${${1}}"
});

makeTest({
    name: 'Replacements use a single pass with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${1}': '${2}', '${2}': '${1}' }
    },
    beforeReplacements: "${1}${2}",
    afterReplacements: "${2}${1}"
});

makeTest({
    name: 'Multiple instances of same substitution string with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${1}': '${2}' }
    },
    beforeReplacements: "{${1}${1}}",
    afterReplacements: "{${2}${2}}"
});

makeTest({
    name: 'Mismatched replacement with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${2}': '${1}' }
    },
    beforeReplacements: "${1}",
    afterReplacements: "${1}"
});

makeTest({
    name: 'Recursive and reduce size with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%1%%': '1' }
    },
    beforeReplacements: "%%%%1%%%%",
    afterReplacements: "%%1%%"
});

makeTest({
    name: 'Recursive and increase size with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%1%%': '%%%%1%%%%' }
    },
    beforeReplacements: "%%1%%",
    afterReplacements: "%%%%1%%%%"
});

makeTest({
    name: 'Replacements use a single pass with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%1%%': '%%2%%', '%%2%%': '%%1%%' }
    },
    beforeReplacements: "%%1%%%%2%%",
    afterReplacements: "%%2%%%%1%%"
});

makeTest({
    name: 'Multiple instances of same substitution string with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%1%%': '%%2%%' }
    },
    beforeReplacements: "%%1%%%%1%%",
    afterReplacements: "%%2%%%%2%%"
});

makeTest({
    name: 'Mismatched replacement with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%2%%': '%%1%%' }
    },
    beforeReplacements: "%%1%%",
    afterReplacements: "%%1%%"
});

makeTest({
    name: 'Case sensativity.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%foo%%': '%%bar%%' }
    },
    beforeReplacements: "%%FOO%%%%foo%%",
    afterReplacements: "%%FOO%%%%bar%%"
});

makeTest({
    name: 'Super macro, a macro with a macro inside it basically, with percents.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '%%%%foo%%%%': 'foo' }
    },
    beforeReplacements: "%%%%foo%%%%",
    afterReplacements: "foo"
});

makeTest({
    name: 'Super macro, with brackets.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${${foo}}': 'foo' }
    },
    beforeReplacements: "${${foo}}",
    afterReplacements: "foo"
});

makeTest({
    name: 'Super macro, with both.',
    auctionConfigOverrides: {
        deprecatedRenderURLReplacements: { '${%%foo%%}': 'foo', '%%${bar}%%':'bar' }
    },
    beforeReplacements: "${%%foo%%}%%${bar}%%",
    afterReplacements: "foobar"
});
