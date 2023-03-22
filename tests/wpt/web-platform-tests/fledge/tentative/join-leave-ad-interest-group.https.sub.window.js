// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js

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

  // "biddingLogicUrl" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicUrl: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicUrl: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicUrl: 'data:text/javascript,Foo' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicUrl: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingLogicUrl: 'relative/path' }
  },

  // "biddingWasmHelperUrl" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperUrl: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperUrl: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperUrl: 'data:application/wasm,Foo' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperUrl: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     biddingWasmHelperUrl: 'relative/path' }
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

  // "trustedBiddingSignalsUrl" tests
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsUrl: null }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsUrl: 'https://{{hosts[][www]}}/foo.js' }
  },
  { expectJoinSucces: false,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsUrl: 'data:application/json,{}' }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsUrl: `${window.location.origin}/foo.js`}
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     trustedBiddingSignalsUrl: 'relative/path' }
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
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{renderUrl: 'https://somewhere.test/'}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{renderUrl: 'https://somewhere.test/'},
                           {renderUrl: 'https://somewhere-else.test/'}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{renderUrl: 'https://somewhere.test/',
                            metadata: null}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     ads: [{renderUrl: 'https://somewhere.test/',
                            metadata: null,
                            someOtherField: 'foo'}] }
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
                     adComponents: {} }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [] }
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
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{renderUrl: 'https://somewhere.test/'}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{renderUrl: 'https://somewhere.test/'},
                                    {renderUrl: 'https://elsewhere.test/'}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{renderUrl: 'https://somewhere.test/',
                                     metadata: null}] }
  },
  { expectJoinSucces: true,
    expectLeaveSucces: true,
    interestGroup: { ...BASE_INTEREST_GROUP,
                     adComponents: [{renderUrl: 'https://somewhere.test/',
                                     metadata: null,
                                     someOtherField: 'foo'}] }
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
];

for (testCase of SIMPLE_JOIN_LEAVE_TEST_CASES) {
  promise_test((async (testCase) => {
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

    let leave_promise = navigator.leaveAdInterestGroup(testCase.interestGroup,
                                                       INTEREST_GROUP_LIFETIME_SECS);
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
  }).bind(undefined, testCase), 'Join and leave interest group: ' + JSON.stringify(testCase));
}

promise_test(async test => {
  const uuid = generateUuid(test);

  // Joining an interest group without a bidding script and run an auction.
  // There should be no winner.
  await joinInterestGroup(test, uuid, { biddingLogicUrl: null });
  assert_equals(null, await runBasicFledgeAuction(test, uuid),
                'Auction unexpectedly had a winner');

  // Joining an interest group with a bidding script and the same owner/name as
  // the previously joined interest group, and re-run the auction. There should
  // be a winner this time.
  await joinInterestGroup(test, uuid);
  let url = await runBasicFledgeAuction(test, uuid);
  assert_true('string' === typeof url,
              'Wrong value type returned from auction: ' + typeof url);

  // Re-join the first interest group, and re-run the auction. The interest
  // group should be overwritten again, and there should be no winner.
  await joinInterestGroup(test, uuid, { biddingLogicUrl: null });
  assert_equals(null, await runBasicFledgeAuction(test, uuid),
                'Auction unexpectedly had a winner');
}, 'Join same interest group overwrites old matching group.');

promise_test(async test => {
  const uuid = generateUuid(test);

  // Join an interest group, run an auction to make sure it was joined.
  await joinInterestGroup(test, uuid);
  let url = await runBasicFledgeAuction(test, uuid);
  assert_true('string' === typeof url,
              'Wrong value type returned from auction: ' + typeof url);

  // Leave the interest group, re-run the auction. There should be no winner.
  await leaveInterestGroup();
  assert_equals(null, await runBasicFledgeAuction(test, uuid),
                'Auction unexpectedly had a winner');
}, 'Leaving interest group actually leaves interest group.');

promise_test(async test => {
  // This should not throw.
  await leaveInterestGroup({ name: 'Never join group' });
}, 'Leave an interest group that was never joined.');
