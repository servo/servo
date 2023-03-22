// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: timeout=long

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');`,
      // reportWin:
      'sellerSignals === null',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      [createSellerReportUrl(uuid), createBidderReportUrl(uuid)]
  );
}, 'Both send reports, seller passes nothing to bidder.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');`,
      // reportWin:
      null,
      '',
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
}, 'Only seller sends a report');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');`,
      // reportWin:
      null,
      'throw new Error("Very serious exception")',
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
}, 'Only seller sends a report, bidder throws an exception');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');`,
      // reportWin:
      null,
      null,
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
}, 'Only seller sends a report, bidder has no reportWin() method');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      '',
      // reportWin:
      'sellerSignals === null',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      'return "foo";',
      // reportWin:
      'sellerSignals === "foo"',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report, seller passes a message to bidder');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      'throw new Error("Very serious exception")',
      // reportWin:
      'sellerSignals === null',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report, seller throws an exception');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      null,
      // reportWin:
      'sellerSignals === null',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Only bidder sends a report, seller has no reportResult() method');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');
       sendReportTo('${createSellerReportUrl(uuid)}');
       return 5;`,
      // reportWin:
      'sellerSignals === null',
      `sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createBidderReportUrl(uuid)]
  );
}, 'Seller calls sendReportTo() twice, which throws an exception.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `sendReportTo('${createSellerReportUrl(uuid)}');`,
      // reportWin:
      null,
      `sendReportTo('${createBidderReportUrl(uuid)}');
       sendReportTo('${createBidderReportUrl(uuid)}');`,
      // expectedReportUrls:
      [createSellerReportUrl(uuid)]
  );
  // Seller reports may be sent before bidder reports, since reportWin()
  // takes output from reportResult() as input. Wait to make sure the
  // bidder report URL really is not being requested.
  await new Promise(resolve => test.step_timeout(resolve, 200));
  await waitForObservedRequests(uuid, [createSellerReportUrl(uuid)]);
}, 'Bidder calls sendReportTo() twice, which throws an exception.');
