// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: timeout=long

"use strict;"

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportUrl(uuid)}');`,
        reportWinSuccessCondition:
          'sellerSignals === null',
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}, 'Both send reports, seller passes nothing to bidder.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportUrl(uuid)}');`,
        reportWin:
          '' },
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
}, 'Only seller sends a report');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportUrl(uuid)}');`,
        reportWin:
          'throw new Error("Very serious exception")' },
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
}, 'Only seller sends a report, bidder throws an exception');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
}, 'Only seller sends a report, bidder has no reportWin() method');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          '',
        reportWinSuccessCondition:
          'sellerSignals === null',
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          'return "foo";',
        reportWinSuccessCondition:
          'sellerSignals === "foo"',
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report, seller passes a message to bidder');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          'throw new Error("Very serious exception")',
        reportWinSuccessCondition:
          'sellerSignals === null',
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report, seller throws an exception');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportWinSuccessCondition:
          'sellerSignals === null',
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report, seller has no reportResult() method');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportUrl(uuid)}');
           sendReportTo('${createSellerReportUrl(uuid)}');
           return 5;`,
        reportWinSuccessCondition:
          'sellerSignals === null',
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Seller calls sendReportTo() twice, which throws an exception.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `sendReportTo('${createSellerReportUrl(uuid)}');`,
        reportWin:
          `sendReportTo('${createBidderReportUrl(uuid)}');
           sendReportTo('${createBidderReportUrl(uuid)}');` },
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
  // Seller reports may be sent before bidder reports, since reportWin()
  // takes output from reportResult() as input. Wait to make sure the
  // bidder report URL really is not being requested.
  await new Promise(resolve => test.step_timeout(resolve, 200));
  await waitForObservedRequests(uuid, [createSellerReportUrl(uuid)]);
}, 'Bidder calls sendReportTo() twice, which throws an exception.');
