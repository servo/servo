// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-10
// META: variant=?11-20
// META: variant=?21-30
// META: variant=?31-40
// META: variant=?41-50
// META: variant=?51-60
// META: variant=?61-70
// META: variant=?71-80
// META: variant=?81-last

"use strict;"

// These tests are focused on joinAdInterestGroup() and leaveAdInterestGroup().
// Most join tests do not run auctions, but instead only check the result of
// the returned promise, since testing that interest groups are actually
// joined, and that each interestGroup field behaves as intended, are covered
// by other tests.

// Minimal fields needed for a valid interest group. Used in most test cases.
const BASE_INTEREST_GROUP = {
  owner: window.location.origin,
  name: 'default name',
}

// Each test case attempts to join and then leave an interest group, checking
// if any exceptions are thrown from either operation.
const SIMPLE_JOIN_LEAVE_TEST_CASES = [
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: null
  },
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: {}
  },

  // Basic success test case.
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: BASE_INTEREST_GROUP
  },

  // "owner" tests
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: { name: 'default name' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     owner: null}
  },
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     owner: window.location.origin.replace('https', 'http')}
  },
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     owner: window.location.origin.replace('https', 'wss')}
  },
  // Cross-origin joins and leaves are not allowed without .well-known
  // permissions.
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     owner: '{{hosts[][www]}}' }
  },

  // "name" tests
  { expectJoinSucces: false,
    expectLeaveSucces: false,
    interestGroup: { owner: window.location.origin }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     name: ''}
  },

  // "priority" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priority: 1}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priority: 0}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priority: -1.5}
  },

  // "priorityVector" tests
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priorityVector: null}
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priorityVector: 1}
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priorityVector: {a: 'apple'}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priorityVector: {}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priorityVector: {a: 1}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     priorityVector: {'a': 1, 'b': -4.5, 'a.b': 0}}
  },

  // "prioritySignalsOverrides" tests
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     prioritySignalsOverrides: null}
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     prioritySignalsOverrides: 1}
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     prioritySignalsOverrides: {a: 'apple'}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     prioritySignalsOverrides: {}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     prioritySignalsOverrides: {a: 1}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     prioritySignalsOverrides: {'a': 1, 'b': -4.5, 'a.b': 0}}
  },

  // "enableBiddingSignalsPrioritization" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     enableBiddingSignalsPrioritization: true}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     enableBiddingSignalsPrioritization: false}
  },

  // "biddingLogicURL" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicURL: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicURL: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicURL: 'data:text/javascript,Foo' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicURL: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicURL: 'relative/path' }
  },

  // "biddingWasmHelperURL" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperURL: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperURL: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperURL: 'data:application/wasm,Foo' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperURL: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperURL: 'relative/path' }
  },

  // "dailyUpdateUrl" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     dailyUpdateUrl: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     dailyUpdateUrl: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     dailyUpdateUrl: 'data:application/wasm,Foo' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     dailyUpdateUrl: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     dailyUpdateUrl: 'relative/path' }
  },

  // "executionMode" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     executionMode: 'compatibility' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     executionMode: 'groupByOrigin' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     executionMode: 'unknownValuesAreValid' }
  },

  // "trustedBiddingSignalsURL" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsURL: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsURL: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsURL: 'data:application/json,{}' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsURL: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsURL: 'relative/path' }
  },

  // "trustedBiddingSignalsKeys" tests
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsKeys: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsKeys: {}}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsKeys: []}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsKeys: ['a', 4, 'Foo']}
  },

  // "userBiddingSignals" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     userBiddingSignals: null }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     userBiddingSignals: 'foo' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     userBiddingSignals: 15 }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     userBiddingSignals: [5, 'foo', [-6.4, {a: 'b'}]] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     userBiddingSignals: {a: [5, 'foo', {b: -6.4}] }}
  },

  // "ads" tests
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: 5 }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: {} }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [] }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{}] }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{metadata: [{a:'b'}, 'c'], 1:[2,3]}] }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{renderURL: 'https://somewhere.test/',
                            adRenderId: 'thirteenChars' }] }
  },

  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{renderURL: 'https://somewhere.test/'}] }
  },

  // "adComponents" tests
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: 5 }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{}] }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{metadata: [{a:'b'}, 'c'], 1:[2,3]}] }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{renderURL: 'https://somewhere.test/',
                                     adRenderId: 'More than twelve characters'}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{renderURL: 'https://somewhere.test/'}] }
  },

  // Miscellaneous tests.
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     extra: false,
                     fields: {do:'not'},
                     matter: 'at',
                     all: [3,4,5] }
  },

  // Interest group dictionaries must be less than 1 MB (1048576 bytes), so
  // test that here by using a large name on an otherwise valid interest group
  // dictionary. The first case is the largest name value that still results in
  // a valid dictionary, whereas the second test case produces a dictionary
  // that's one byte too large.
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
      name: 'a'.repeat(1048516)
    },
    testCaseName: "Largest possible interest group dictionary",
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
      name: 'a'.repeat(1048517)
    },
    testCaseName: "Oversized interest group dictionary",
  },
];

for (testCase of SIMPLE_JOIN_LEAVE_TEST_CASES) {
  var test_name = 'Join and leave interest group: ';
  if ('testCaseName' in testCase) {
    test_name += testCase.testCaseName;
  } else {
    test_name += JSON.stringify(testCase);
  }

  subsetTest(promise_test, (async (testCase) => {
    const INTEREST_GROUP_LIFETIME_SECS = 1;

    let join_promise = navigator.joinAdInterestGroup(testCase.interestGroup,
                                                     INTEREST_GROUP_LIFETIME_SECS);
    assert_true(join_promise instanceof Promise, "join should return a promise");
    if (testCase.expectJoinSucces) {
      assert_equals(await join_promise, undefined);
    } else {
      let joinExceptionThrown = false;
      try {
        await join_promise;
      } catch (e) {
        joinExceptionThrown = true;
      }
      assert_true(joinExceptionThrown, 'Exception not thrown on join.');
    }

    let leave_promise = navigator.leaveAdInterestGroup(testCase.interestGroup);
    assert_true(leave_promise instanceof Promise, "leave should return a promise");
    if (testCase.expectLeaveSucces) {
      assert_equals(await leave_promise, undefined);
    } else {
      let leaveExceptionThrown = false;
      try {
        await leave_promise;
      } catch (e) {
        leaveExceptionThrown = true;
      }
      assert_true(leaveExceptionThrown, 'Exception not thrown on leave.');
    }
  }).bind(undefined, testCase), test_name);
}

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Joining an interest group without a bidding script and run an auction.
  // There should be no winner.
  await joinInterestGroup(test, uuid, { biddingLogicURL: null });
  assert_equals(null, await runBasicFledgeAuction(test, uuid),
                'Auction unexpectedly had a winner');

  // Joining an interest group with a bidding script and the same owner/name as
  // the previously joined interest group, and re-run the auction. There should
  // be a winner this time.
  await joinInterestGroup(test, uuid);
  let config = await runBasicFledgeAuction(test, uuid);
  assert_true(config instanceof FencedFrameConfig,
              'Wrong value type returned from auction: ' +
              config.constructor.name);

  // Re-join the first interest group, and re-run the auction. The interest
  // group should be overwritten again, and there should be no winner.
  await joinInterestGroup(test, uuid, { biddingLogicURL: null });
  assert_equals(null, await runBasicFledgeAuction(test, uuid),
                'Auction unexpectedly had a winner');
}, 'Join same interest group overwrites old matching group.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join an interest group, run an auction to make sure it was joined.
  await joinInterestGroup(test, uuid);
  let config = await runBasicFledgeAuction(test, uuid);
  assert_true(config instanceof FencedFrameConfig,
              'Wrong value type returned from auction: ' +
              config.constructor.name);

  // Leave the interest group, re-run the auction. There should be no winner.
  await leaveInterestGroup();
  assert_equals(null, await runBasicFledgeAuction(test, uuid),
                'Auction unexpectedly had a winner');
}, 'Leaving interest group actually leaves interest group.');

subsetTest(promise_test, async test => {
  // This should not throw.
  await leaveInterestGroup({ name: 'Never join group' });
}, 'Leave an interest group that was never joined.');

///////////////////////////////////////////////////////////////////////////////
// Expiration tests
///////////////////////////////////////////////////////////////////////////////

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Joins the default interest group, with a 0.2 second duration.
  await joinInterestGroup(test, uuid, {}, 0.2);

  // Keep on running auctions until interest group duration expires.
  // Unfortunately, there's no duration that's guaranteed to be long enough to
  // be be able to win an auction once, but short enough to prevent this test
  // from running too long, so can't check the interest group won at least one
  // auction.
  while (await runBasicFledgeAuction(test, uuid) !== null);
}, 'Interest group duration.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join interest group with a duration of -600. The interest group should
  // immediately expire, and not be allowed to participate in auctions.
  await joinInterestGroup(test, uuid, {}, -600);
  assert_true(await runBasicFledgeAuction(test, uuid) === null);
}, 'Interest group duration of -600.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join a long-lived interest group.
  await joinInterestGroup(test, uuid, {}, 600);

  // Make sure interest group with a non-default timeout was joined.
  assert_true(await runBasicFledgeAuction(test, uuid) !== null);

  // Re-join interest group with a duration value of 0.2 seconds.
  await joinInterestGroup(test, uuid, {}, 0.2);

  // Keep on running auctions until interest group expires.
  while (await runBasicFledgeAuction(test, uuid) !== null);
}, 'Interest group test with overwritten duration.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);

  // Join a long-lived interest group.
  await joinInterestGroup(test, uuid, {}, 600);

  // Re-join interest group with a duration value of 0.2 seconds. The new
  // duration should take precedence, and the interest group should immediately
  // expire.
  await joinInterestGroup(test, uuid, {}, -600);
  assert_true(await runBasicFledgeAuction(test, uuid) === null);
}, 'Interest group test with overwritten duration of -600.');
