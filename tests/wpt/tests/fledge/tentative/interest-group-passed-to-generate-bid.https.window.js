// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-15
// META: variant=?16-last

"use strict;"

// These tests focus on making sure InterestGroup fields are passed to generateBid(),
// and are normalized if necessary. This test does not check the behaviors of the
// fields.

const makeTest = ({
  // Test name.
  name,
  // InterestGroup field name.
  fieldName,
  // InterestGroup field value, both expected in worklets and acution in the
  // auction. If undefined, value will not be set in interestGroup, and will
  // be expected to also not be set in the interestGroup passed to generateBid().
  fieldValue,
  // Additional values to use in the InterestGroup passed to joinInterestGroup().
  // If it contains a value for the key specified in `fieldName`, that takes
  // precedent over `fieldValue`.
  interestGroupOverrides = {}
}) => {
  subsetTest(promise_test, async test => {
    const uuid = generateUuid(test);

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
                `if (!${comparison})
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
  interestGroupOverrides: {seller: ` ${OTHER_ORIGIN1.toUpperCase()} `}
});

makeTest({
  name: 'InterestGroup.owner is URL.',
  fieldName: 'owner',
  fieldValue: OTHER_ORIGIN1,
  interestGroupOverrides: {seller: OTHER_ORIGIN1 + "/Foopy"}
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL not set.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: undefined
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: `${OTHER_ORIGIN1}${BASE_PATH}this-file-does-not-exist.json`,
  interestGroupOverrides: {owner: OTHER_ORIGIN1}
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsURL with non-normalized value.',
  fieldName: 'trustedBiddingSignalsURL',
  fieldValue: `${OTHER_ORIGIN1}${BASE_PATH}this-file-does-not-exist.json`,
  interestGroupOverrides: {
    owner: OTHER_ORIGIN1,
    trustedScoringSignalsURL:
        `${OTHER_ORIGIN1.toUpperCase()}${BASE_PATH}this-file-does-not-exist.json`
  }
});

makeTest({
  name: 'InterestGroup.trustedBiddingSignalsKeys not set.',
  fieldName: 'trustedBiddingSignalsKeys',
  fieldValue: undefined
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
  name: 'InterestGroup.nonStandardField.',
  fieldName: 'nonStandardField',
  fieldValue: undefined,
  interestGroupOverrides: {nonStandardField: 'This value should not be passed to worklets'}
});
