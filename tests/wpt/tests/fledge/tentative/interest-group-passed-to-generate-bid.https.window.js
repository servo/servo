// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-20
// META: variant=?21-25
// META: variant=?26-30
// META: variant=?31-35
// META: variant=?36-40
// META: variant=?41-45
// META: variant=?46-50
// META: variant=?51-55
// META: variant=?56-60
// META: variant=?61-65
// META: variant=?66-70
// META: variant=?71-75
// META: variant=?76-80
// META: variant=?81-85

"use strict;"

// These tests focus on making sure InterestGroup fields are passed to generateBid(),
// and are normalized if necessary. This test does not check the behaviors of the
// fields.

// Modifies "ads". Replaces "REPLACE_WITH_UUID" in all "renderURL" fields of
// objects in "ads" array with "uuid". Generated ad URLs have embedded
// UUIDs to prevent InterestGroups unexpectedly left over from one test from
// messing up another test, but these tests need ad URLs before the UUID is
// generated. To get around that, "REPLACE_WITH_UUID" is used in place of UUIDs
// and then this is used to replace them with the real UUID.
function updateAdRenderURLs(ads, uuid) {
  for (let i = 0; i < ads.length; ++i) {
    let ad = ads[i];
    ad.renderURL = ad.renderURL.replace('REPLACE_WITH_UUID', uuid);
  }
}

const makeTest = ({
  // Test name.
  name,
  // InterestGroup field name.
  fieldName,
  // InterestGroup field value, both expected in worklets and the value used
  // when joining the interest group. If undefined, value will not be set in
  // interestGroup, and will be expected to also not be set in the
  // interestGroup passed to generateBid().
  fieldValue,
  // Additional values to use in the InterestGroup passed to joinInterestGroup().
  // If it contains a value for the key specified in `fieldName`, takes
  // precedent over `fieldValue`.
  interestGroupOverrides = {}
}) => {
  subsetTest(promise_test, async test => {
    const uuid = generateUuid(test);

    // It's not strictly necessary to replace UUIDs in "adComponents", but do it for consistency.
    if (fieldName === 'ads' || fieldName === 'adComponents' && fieldValue) {
      updateAdRenderURLs(fieldValue, uuid);
    }

    if (interestGroupOverrides.ads) {
      updateAdRenderURLs(interestGroupOverrides.ads, uuid);
    }

    if (interestGroupOverrides.adComponents) {
      updateAdRenderURLs(interestGroupOverrides.adComponents, uuid);
    }

    if (!(fieldName in interestGroupOverrides) && fieldValue !== undefined)
      interestGroupOverrides[fieldName] = fieldValue;

    let comparison = `deepEquals(interestGroup["${fieldName}"], ${JSON.stringify(fieldValue)})`;
    // In the case it's undefined, require value not to be set.
    if (fieldValue === undefined)
      comparison = `!("${fieldName}" in interestGroup)`;

    // Prefer to use `interestGroupOverrides.owner` if present. Treat it as a URL
    // and then convert it to an origin because one test passes in a URL.
    let origin = location.origin;
    if (interestGroupOverrides.owner)
      origin = new URL(interestGroupOverrides.owner).origin;

    interestGroupOverrides.biddingLogicURL =
        createBiddingScriptURL(
            { origin: origin,
              generateBid:
                  `// Delete deprecated "renderUrl" fields from ads and adComponents, if
                  // present.
                  for (let field in interestGroup) {
                    if (field === "ads" || field === "adComponents") {
                      for (let i = 0; i < interestGroup[field].length; ++i) {
                        let ad = interestGroup[field][i];
                        delete ad.renderUrl;
                      }
                    }
                  }
                  if (!${comparison})
                    throw "Unexpected value: " + JSON.stringify(interestGroup["${fieldName}"]);`
            });
    if (origin !== location.origin) {
      await joinCrossOriginInterestGroup(test, uuid, origin, interestGroupOverrides);
    } else {
      await joinInterestGroup(test, uuid, interestGroupOverrides);
    }

    await runBasicFledgeTestExpectingWinner(test, uuid, {interestGroupBuyers: [origin]});
  }, name);
};

makeTest({
  name: 'InterestGroup.owner.',
  fieldName: 'owner',
  fieldValue: OTHER_ORIGIN1
});

makeTest({
  name: 'InterestGroup.owner with non-normalized origin.',
  fieldName: 'owner',
  fieldValue: OTHER_ORIGIN1,
  interestGroupOverrides: {owner: ` ${OTHER_ORIGIN1.toUpperCase()} `}
});

makeTest({
  name: 'InterestGroup.owner is URL.',
  fieldName: 'owner',
  fieldValue: OTHER_ORIGIN1,
  interestGroupOverrides: {owner: OTHER_ORIGIN1 + '/Foopy'}
});

makeTest({
  name: 'InterestGroup.name.',
  fieldName: 'name',
  fieldValue: 'Jim'
});

makeTest({
  name: 'InterestGroup.name with unicode characters.',
  fieldName: 'name',
  fieldValue: '\u2665'
});

makeTest({
  name: 'InterestGroup.name with empty name.',
  fieldName: 'name',
  fieldValue: ''
});

makeTest({
  name: 'InterestGroup.name with unpaired surrogate characters, which should be replaced with "\\uFFFD".',
  fieldName: 'name',
  fieldValue: '\uFFFD,\uFFFD',
  interestGroupOverrides: {name: '\uD800,\uDBF0'}
});

// Since "biddingLogicURL" contains the script itself inline, can't include the entire URL
// in the script for an equality check. Instead, replace the "generateBid" query parameter
// in the URL with an empty value before comparing it. This doesn't just delete the entire
// query parameter to make sure that's correctly passed in.
subsetTest(promise_test,async test => {
  const uuid = generateUuid(test);

  let biddingScriptBaseURL = createBiddingScriptURL({origin: OTHER_ORIGIN1, generateBid: ''});
  let biddingLogicURL = createBiddingScriptURL(
      { origin: OTHER_ORIGIN1,
        generateBid:
          `let biddingScriptBaseURL =
            interestGroup.biddingLogicURL.replace(/generateBid=[^&]*/, "generateBid=");
          if (biddingScriptBaseURL !== "${biddingScriptBaseURL}")
            throw "Wrong bidding script URL: " + interestGroup.biddingLogicURL`
      });

  await joinCrossOriginInterestGroup(test, uuid, OTHER_ORIGIN1,
                                     { biddingLogicURL: biddingLogicURL });

  await runBasicFledgeTestExpectingWinner(test, uuid, {interestGroupBuyers: [OTHER_ORIGIN1]});
}, 'InterestGroup.biddingLogicURL.');

// Much like above test, but use a relative URL that points to bidding script.
subsetTest(promise_test,async test => {
  const uuid = generateUuid(test);

  let biddingScriptBaseURL = createBiddingScriptURL({generateBid: ''});
  let biddingLogicURL = createBiddingScriptURL(
      { generateBid:
          `let biddingScriptBaseURL =
            interestGroup.biddingLogicURL.replace(/generateBid=[^&]*/, "generateBid=");
          if (biddingScriptBaseURL !== "${biddingScriptBaseURL}")
            throw "Wrong bidding script URL: " + interestGroup.biddingLogicURL`
      });
  biddingLogicURL = biddingLogicURL.replace(BASE_URL, 'foo/../');

  await joinInterestGroup(test, uuid, { biddingLogicURL: biddingLogicURL });

  await runBasicFledgeTestExpectingWinner(test, uuid);
}, 'InterestGroup.biddingLogicURL with relative URL.');

makeTest({
  name: 'InterestGroup.lifetimeMs should not be passed in.',
  fieldName: 'lifetimeMs',
  fieldValue: undefined,
  interestGroupOverrides: { lifetimeMs: "120000" }
});

makeTest({
  name: 'InterestGroup.priority should not be passed in, since it can be changed by auctions.',
  fieldName: 'priority',
  fieldValue: undefined,
  interestGroupOverrides: { priority: 500 }
});

makeTest({
  name: 'InterestGroup.priorityVector undefined.',
  fieldName: 'priorityVector',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.priorityVector empty.',
  fieldName: 'priorityVector',
  fieldValue: {}
});

makeTest({
  name: 'InterestGroup.priorityVector.',
  fieldName: 'priorityVector',
  fieldValue: { 'a': -1, 'b': 2 }
});

// TODO: This is currently using USVString internally, so doesn't allow unpaired
// surrogates, but the spec says it should.
makeTest({
  name: 'InterestGroup.priorityVector with unpaired surrogate character.',
  fieldName: 'priorityVector',
  fieldValue: { '\uFFFD': -1 },
  interestGroupOverrides: { prioritySignalsOverrides: { '\uD800': -1 } }
});

makeTest({
  name: 'InterestGroup.prioritySignalsOverrides should not be passed in, since it can be changed by auctions.',
  fieldName: 'prioritySignalsOverrides',
  fieldValue: undefined,
  interestGroupOverrides: { prioritySignalsOverrides: { 'a': 1, 'b': 2 } }
});

makeTest({
  name: 'InterestGroup.enableBiddingSignalsPrioritization not set.',
  fieldName: 'enableBiddingSignalsPrioritization',
  fieldValue: false,
  interestGroupOverrides: { enableBiddingSignalsPrioritization: undefined }
});

makeTest({
  name: 'InterestGroup.enableBiddingSignalsPrioritization unrecognized.',
  fieldName: 'enableBiddingSignalsPrioritization',
  // Non-empty strings are treated as true by Javascript. This test is serves
  // to make sure that the 'foo' isn't preserved.
  fieldValue: true,
  interestGroupOverrides: { enableBiddingSignalsPrioritization: 'foo' }
});

makeTest({
  name: 'InterestGroup.enableBiddingSignalsPrioritization false.',
  fieldName: 'enableBiddingSignalsPrioritization',
  fieldValue: false
});

makeTest({
  name: 'InterestGroup.enableBiddingSignalsPrioritization true.',
  fieldName: 'enableBiddingSignalsPrioritization',
  fieldValue: true
});

makeTest({
  name: 'InterestGroup.biddingWasmHelperURL not set.',
  fieldName: 'biddingWasmHelperURL',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.biddingWasmHelperURL.',
  fieldName: 'biddingWasmHelperURL',
  fieldValue: `${OTHER_ORIGIN1}${RESOURCE_PATH}wasm-helper.py`,
  interestGroupOverrides: {owner: OTHER_ORIGIN1}
});

makeTest({
  name: 'InterestGroup.biddingWasmHelperURL with non-normalized value.',
  fieldName: 'biddingWasmHelperURL',
  fieldValue: `${OTHER_ORIGIN1}${RESOURCE_PATH}wasm-helper.py`,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    biddingWasmHelperURL:
        `${OTHER_ORIGIN1.toUpperCase()}${RESOURCE_PATH}wasm-helper.py`
  }
});

makeTest({
  name: 'InterestGroup.biddingWasmHelperURL with relative URL.',
  fieldName: 'biddingWasmHelperURL',
  fieldValue: `${OTHER_ORIGIN1}${RESOURCE_PATH}wasm-helper.py`,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    biddingWasmHelperURL: 'foo/../resources/wasm-helper.py'
  }
});

makeTest({
  name: 'InterestGroup.biddingWasmHelperURL with unpaired surrogate characters, which should be replaced with "\\uFFFD".',
  fieldName: 'biddingWasmHelperURL',
  fieldValue: (new URL(`${OTHER_ORIGIN1}${RESOURCE_PATH}wasm-helper.py?\uFFFD.\uFFFD`)).href,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    biddingWasmHelperURL: `${OTHER_ORIGIN1}${RESOURCE_PATH}wasm-helper.py?\uD800.\uDBF0`
  }
});

makeTest({
  name: 'InterestGroup.updateURL not set.',
  fieldName: 'updateURL',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.updateURL.',
  fieldName: 'updateURL',
  fieldValue: `${OTHER_ORIGIN1}${BASE_PATH}This-File-Does-Not-Exist.json`,
  interestGroupOverrides: {owner: OTHER_ORIGIN1}
});

makeTest({
  name: 'InterestGroup.updateURL with non-normalized value.',
  fieldName: 'updateURL',
  fieldValue: `${OTHER_ORIGIN1}${BASE_PATH}This-File-Does-Not-Exist.json`,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    updateURL: `${OTHER_ORIGIN1.toUpperCase()}${BASE_PATH}This-File-Does-Not-Exist.json`
  }
});

makeTest({
  name: 'InterestGroup.updateURL with relative URL.',
  fieldName: 'updateURL',
  fieldValue: (new URL(`${OTHER_ORIGIN1}${BASE_PATH}../This-File-Does-Not-Exist.json`)).href,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    updateURL: '../This-File-Does-Not-Exist.json'
  }
});

makeTest({
  name: 'InterestGroup.updateURL with unpaired surrogate characters, which should be replaced with "\\uFFFD".',
  fieldName: 'updateURL',
  fieldValue: (new URL(`${BASE_URL}\uFFFD.\uFFFD`)).href,
  interestGroupOverrides: {
    updateURL: `${BASE_URL}\uD800.\uDBF0`
  }
});

makeTest({
  name: 'InterestGroup.executionMode not present.',
  fieldName: 'executionMode',
  fieldValue: 'compatibility',
  interestGroupOverrides: { executionMode: undefined }
});

makeTest({
  name: 'InterestGroup.executionMode compatibility.',
  fieldName: 'executionMode',
  fieldValue: 'compatibility'
});

makeTest({
  name: 'InterestGroup.executionMode frozen-context.',
  fieldName: 'executionMode',
  fieldValue: 'frozen-context'
});

makeTest({
  name: 'InterestGroup.executionMode group-by-origin.',
  fieldName: 'executionMode',
  fieldValue: 'group-by-origin'
});

makeTest({
  name: 'InterestGroup.executionMode has non-standard string.',
  fieldName: 'executionMode',
  fieldValue: 'compatibility',
  interestGroupOverrides: { executionMode: 'foo' }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL not set.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: `${OTHER_ORIGIN1}${BASE_PATH}This-File-Does-Not-Exist.json`,
  interestGroupOverrides: {owner: OTHER_ORIGIN1}
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL with non-normalized value.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: `${OTHER_ORIGIN1}${BASE_PATH}This-File-Does-Not-Exist.json`,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    trustedBiddingSignalsURL:
        `${OTHER_ORIGIN1.toUpperCase()}${BASE_PATH}This-File-Does-Not-Exist.json`
  }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL with relative URL.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: (new URL(`${OTHER_ORIGIN1}${BASE_PATH}../This-File-Does-Not-Exist.json`)).href,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    trustedBiddingSignalsURL: '../This-File-Does-Not-Exist.json'
  }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL with unpaired surrogate characters, which should be replaced with "\\uFFFD".',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: (new URL(`${BASE_URL}\uFFFD.\uFFFD`)).href,
  interestGroupOverrides: {
    trustedBiddingSignalsURL: `${BASE_URL}\uD800.\uDBF0`
  }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsKeys not set.',
  fieldName: 'trustedBiddingSignalsKeys',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsKeys.',
  fieldName: 'trustedBiddingSignalsKeys',
  fieldValue: ['a', ' b ', 'c', '1', '%20', '3', '\u2665']
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsKeys with non-normalized values.',
  fieldName: 'trustedBiddingSignalsKeys',
  fieldValue: ['1', '2', '3'],
  interestGroupOverrides: { trustedBiddingSignalsKeys: [1, 0x2, '3'] }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsKeys unpaired surrogate characters, which should be replaced with "\\uFFFD".',
  fieldName: 'trustedBiddingSignalsKeys',
  fieldValue: ['\uFFFD', '\uFFFD', '\uFFFD.\uFFFD'],
  interestGroupOverrides: { trustedBiddingSignalsKeys: ['\uD800', '\uDBF0', '\uD800.\uDBF0'] }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsSlotSizeMode empty.',
  fieldName: 'trustedBiddingSignalsSlotSizeMode',
  fieldValue: 'none',
  interestGroupOverrides: { trustedBiddingSignalsSlotSizeMode: undefined }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsSlotSizeMode none.',
  fieldName: 'trustedBiddingSignalsSlotSizeMode',
  fieldValue: 'none'
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsSlotSizeMode slot-size.',
  fieldName: 'trustedBiddingSignalsSlotSizeMode',
  fieldValue: 'slot-size'
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsSlotSizeMode all-slots-requested-sizes.',
  fieldName: 'trustedBiddingSignalsSlotSizeMode',
  fieldValue: 'all-slots-requested-sizes'
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsSlotSizeMode unrecognized value.',
  fieldName: 'trustedBiddingSignalsSlotSizeMode',
  fieldValue: 'none',
  interestGroupOverrides: { trustedBiddingSignalsSlotSizeMode: 'unrecognized value' }
});

makeTest({
  name: 'InterestGroup.userBiddingSignals not set.',
  fieldName: 'userBiddingSignals',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.userBiddingSignals is integer.',
  fieldName: 'userBiddingSignals',
  fieldValue: 15
});

makeTest({
  name: 'InterestGroup.userBiddingSignals is array.',
  fieldName: 'userBiddingSignals',
  fieldValue: [1, {a: 'b'}, 'c']
});

makeTest({
  name: 'InterestGroup.userBiddingSignals is object.',
  fieldName: 'userBiddingSignals',
  fieldValue: {a:1, b:32.5, c:['d', 'e']}
});

makeTest({
  name: 'InterestGroup.userBiddingSignals unpaired surrogate characters, which should be kept as-is.',
  fieldName: 'userBiddingSignals',
  fieldValue: '\uD800.\uDBF0'
});

makeTest({
  name: 'InterestGroup.userBiddingSignals unpaired surrogate characters in an object, which should be kept as-is.',
  fieldName: 'userBiddingSignals',
  fieldValue: {'\uD800': '\uDBF0', '\uDBF0':['\uD800']}
});

makeTest({
  name: 'InterestGroup.nonStandardField.',
  fieldName: 'nonStandardField',
  fieldValue: undefined,
  interestGroupOverrides: {nonStandardField: 'This value should not be passed to worklets'}
});

// Note that all ad tests have a deprecated "renderUrl" field passed to generateBid.

// Ad URLs need the right UUID for seller scripts to accept their bids. Since UUID changes
// for each test, and is not available outside makeTest(), have to use string that will
// be replaced with the real UUID.
const AD1_URL = createRenderURL('REPLACE_WITH_UUID', /*script=*/';');
const AD2_URL = createRenderURL('REPLACE_WITH_UUID', /*script=*/';;');

makeTest({
  name: 'InterestGroup.ads with one ad.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL}]
});

makeTest({
  name: 'InterestGroup.ads one ad with metadata object.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL, metadata: {foo: 1, bar: [2, 3], baz: '4'}}]
});

makeTest({
  name: 'InterestGroup.ads one ad with metadata string.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL, metadata: 'foo'}]
});

makeTest({
  name: 'InterestGroup.ads one ad with null metadata.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL, metadata: null}]
});

makeTest({
  name: 'InterestGroup.ads one ad with adRenderId. This field should not be passed to generateBid.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {ads: [{renderURL: AD1_URL, adRenderId: 'twelve chars'}]}
});

makeTest({
  name: 'InterestGroup.ads one ad with buyerAndSellerReportingId. This field should not be passed to generateBid.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {ads: [{renderURL: AD1_URL,
                                  buyerAndSellerReportingId: 'Arbitrary text'}]}
});

makeTest({
  name: 'InterestGroup.ads one ad with buyerReportingId. This field should not be passed to generateBid.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {ads: [{renderURL: AD1_URL,
                                  buyerReportingId: 'Arbitrary text'}]}
});

makeTest({
  name: 'InterestGroup.ads one ad with novel field. This field should not be passed to generateBid.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {ads: [{renderURL: AD1_URL, novelField: 'Foo'}]}
});

makeTest({
  name: 'InterestGroup.ads with multiple ads.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL, metadata: 1},
               {renderURL: AD2_URL, metadata: [2]}],
  interestGroupOverrides: {ads: [{renderURL: AD1_URL, metadata: 1},
                                 {renderURL: AD2_URL, metadata: [2]}]}
});

// This should probably be an error. This WPT test serves to encourage there to be a
// new join-leave WPT test when that is fixed.
makeTest({
  name: 'InterestGroup.ads duplicate ad.',
  fieldName: 'ads',
  fieldValue: [{renderURL: AD1_URL}, {renderURL: AD1_URL}],
  interestGroupOverrides: {ads: [{renderURL: AD1_URL}, {renderURL: AD1_URL}]}
});

makeTest({
  name: 'InterestGroup.adComponents is undefined.',
  fieldName: 'adComponents',
  fieldValue: undefined
});

// This one is likely a bug.
makeTest({
  name: 'InterestGroup.adComponents is empty array.',
  fieldName: 'adComponents',
  fieldValue: undefined,
  interestGroupOverrides: {adComponents: []}
});

makeTest({
  name: 'InterestGroup.adComponents with one ad.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL}]
});

makeTest({
  name: 'InterestGroup.adComponents one ad with metadata object.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL, metadata: {foo: 1, bar: [2, 3], baz: '4'}}]
});

makeTest({
  name: 'InterestGroup.adComponents one ad with metadata string.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL, metadata: 'foo'}]
});

makeTest({
  name: 'InterestGroup.adComponents one ad with null metadata.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL, metadata: null}]
});

makeTest({
  name: 'InterestGroup.adComponents one ad with adRenderId. This field should not be passed to generateBid.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {adComponents: [{renderURL: AD1_URL,
                                           adRenderId: 'twelve chars'}]}
});

makeTest({
  name: 'InterestGroup.adComponents one ad with buyerAndSellerReportingId. This field should not be passed to generateBid.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {adComponents: [{renderURL: AD1_URL,
                                           buyerAndSellerReportingId: 'Arbitrary text'}]}
});

makeTest({
  name: 'InterestGroup.adComponents one ad with buyerReportingId. This field should not be passed to generateBid.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {adComponents: [{renderURL: AD1_URL,
                                           buyerReportingId: 'Arbitrary text'}]}
});

makeTest({
  name: 'InterestGroup.adComponents one ad with novel field. This field should not be passed to generateBid.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL}],
  interestGroupOverrides: {adComponents: [{renderURL: AD1_URL,
                                           novelField: 'Foo'}]}
});

makeTest({
  name: 'InterestGroup.adComponents with multiple ads.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL, metadata: 1}, {renderURL: AD2_URL, metadata: [2]}]
});

makeTest({
  name: 'InterestGroup.auctionServerRequestFlags is undefined',
  fieldName: 'auctionServerRequestFlags',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.auctionServerRequestFlags is "omit-ads".',
  fieldName: 'auctionServerRequestFlags',
  fieldValue: undefined,
  interestGroupOverrides: {auctionServerRequestFlags: ['omit-ads']}
});

makeTest({
  name: 'InterestGroup.auctionServerRequestFlags is "include-full-ads".',
  fieldName: 'auctionServerRequestFlags',
  fieldValue: undefined,
  interestGroupOverrides: {auctionServerRequestFlags: ['include-full-ads']}
});

makeTest({
  name: 'InterestGroup.auctionServerRequestFlags has multiple values.',
  fieldName: 'auctionServerRequestFlags',
  fieldValue: undefined,
  interestGroupOverrides: {auctionServerRequestFlags: ['omit-ads', 'include-full-ads']}
});

makeTest({
  name: 'InterestGroup.auctionServerRequestFlags.',
  fieldName: 'auctionServerRequestFlags',
  fieldValue: undefined,
  interestGroupOverrides: {auctionServerRequestFlags: ['noval value']}
});

// This should probably be an error. This WPT test serves to encourage there to be a
// new join-leave WPT test when that is fixed.
makeTest({
  name: 'InterestGroup.adComponents duplicate ad.',
  fieldName: 'adComponents',
  fieldValue: [{renderURL: AD1_URL}, {renderURL: AD1_URL}],
  interestGroupOverrides: {adComponents: [{renderURL: AD1_URL}, {renderURL: AD1_URL}]}
});
