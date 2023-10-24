// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: script=/common/subset-tests.js
// META: timeout=long
// META: variant=?1-5
// META: variant=?6-10
// META: variant=?11-last

"use strict;"

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});`,
        reportWin:
          '' },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid)}, body: `],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "",
              destination: ["seller"]
          });`)
  );
}, 'Seller calls registerAdBeacon().');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          '',
        reportWin:
          `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});`
      },
      // expectedReportUrls:
      [`${createBidderBeaconURL(uuid)}, body: `],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "",
              destination: ["buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon().');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});`,
        reportWin:
          '' },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller"]
          });`)
  );
}, 'Seller calls registerAdBeacon(), beacon sent with body.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          '',
        reportWin:
          `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` },
      // expectedReportUrls:
      [`${createBidderBeaconURL(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon(), beacon sent with body.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});`,
        reportWin:
          '' },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid)}, body: body1`,
      `${createSellerBeaconURL(uuid)}, body: body2`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body1",
              destination: ["seller"]
          });
          window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body2",
              destination: ["seller"]
          });`)
  );
}, 'Seller calls registerAdBeacon(). reportEvent() called twice.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          '',
        reportWin:
          `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` },
      // expectedReportUrls:
      [`${createBidderBeaconURL(uuid)}, body: body1`,
       `${createBidderBeaconURL(uuid)}, body: body2`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body1",
              destination: ["buyer"]
          });
          window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body2",
              destination: ["buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon(). reportEvent() called twice.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
        `registerAdBeacon({beacon1: '${createSellerBeaconURL(uuid, '1')}',
                             beacon2: '${createSellerBeaconURL(uuid, '2')}'});`,
        reportWin:
          '' },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid, '1')}, body: body1`,
       `${createSellerBeaconURL(uuid, '2')}, body: body2`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon1",
              eventData: "body1",
              destination: ["seller"]
          });
          window.fence.reportEvent({
              eventType: "beacon2",
              eventData: "body2",
              destination: ["seller"]
          });`)
  );
}, 'Seller calls registerAdBeacon() with multiple beacons.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          '',
        reportWin:
          `registerAdBeacon({beacon1: '${createBidderBeaconURL(uuid, '1')}',
                             beacon2: '${createBidderBeaconURL(uuid, '2')}'});`
      },
      // expectedReportUrls:
      [`${createBidderBeaconURL(uuid, '1')}, body: body1`,
       `${createBidderBeaconURL(uuid, '2')}, body: body2`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon1",
              eventData: "body1",
              destination: ["buyer"]
          });
          window.fence.reportEvent({
              eventType: "beacon2",
              eventData: "body2",
              destination: ["buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon() with multiple beacons.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});`,
        reportWin:
          `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid)}, body: body`,
       `${createBidderBeaconURL(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller","buyer"]
          });`)
  );
}, 'Seller and buyer call registerAdBeacon() with shared reportEvent() call.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});`,
        reportWin:
          `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid)}, body: body1`,
       `${createBidderBeaconURL(uuid)}, body: body2`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
            eventType: "beacon",
            eventData: "body1",
            destination: ["seller"]
          });
          window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body2",
              destination: ["buyer"]
          });`)
  );
}, 'Seller and buyer call registerAdBeacon() with separate reportEvent() calls.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          // Multiple registerAdBeacon() call should result in an exception,
          // throwing away all beacons and other types of reports.
          `sendReportTo('${createSellerReportURL(uuid)}');
           registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});
           registerAdBeacon({beacon1: '${createSellerBeaconURL(uuid)}'});`,
        reportWinSuccessCondition:
          'sellerSignals === null',
        reportWin:
          `registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});` },
      // expectedReportUrls:
      [`${createBidderBeaconURL(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller","buyer"]
          });`)
  );
}, 'Seller calls registerAdBeacon() multiple times.');

subsetTest(promise_test, async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      { reportResult:
          `registerAdBeacon({beacon: '${createSellerBeaconURL(uuid)}'});`,
        reportWin:
          // Multiple registerAdBeacon() call should result in an exception,
          // throwing away all beacons and other types of reports.
          `sendReportTo('${createBidderReportURL(uuid)}');
           registerAdBeacon({beacon: '${createBidderBeaconURL(uuid)}'});
           registerAdBeacon({beacon1: '${createBidderBeaconURL(uuid)}'});` },
      // expectedReportUrls:
      [`${createSellerBeaconURL(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderURL(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller","buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon() multiple times.');
