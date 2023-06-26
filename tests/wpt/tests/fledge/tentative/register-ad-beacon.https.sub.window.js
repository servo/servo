// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.js
// META: timeout=long

"use strict;"

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      null,
      '',
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid)}, body: `],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "",
              destination: ["seller"]
          });`)
  );
}, 'Seller calls registerAdBeacon().');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      '',
      // reportWin:
      null,
      `registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createBidderBeaconUrl(uuid)}, body: `],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "",
              destination: ["buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon().');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      null,
      '',
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller"]
          });`)
  );
}, 'Seller calls registerAdBeacon(), beacon sent with body.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      '',
      // reportWin:
      null,
      `registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createBidderBeaconUrl(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon(), beacon sent with body.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      null,
      '',
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid)}, body: body1`,
       `${createSellerBeaconUrl(uuid)}, body: body2`],
      // renderUrlOverride:
      createRenderUrl(
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

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      '',
      // reportWin:
      null,
      `registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createBidderBeaconUrl(uuid)}, body: body1`,
       `${createBidderBeaconUrl(uuid)}, body: body2`],
      // renderUrlOverride:
      createRenderUrl(
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

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon1: '${createSellerBeaconUrl(uuid, '1')}',
                         beacon2: '${createSellerBeaconUrl(uuid, '2')}'});`,
      // reportWin:
      null,
      '',
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid, '1')}, body: body1`,
       `${createSellerBeaconUrl(uuid, '2')}, body: body2`],
      // renderUrlOverride:
      createRenderUrl(
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

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      '',
      // reportWin:
      null,
      `registerAdBeacon({beacon1: '${createBidderBeaconUrl(uuid, '1')}',
                         beacon2: '${createBidderBeaconUrl(uuid, '2')}'});`,
      // expectedReportUrls:
      [`${createBidderBeaconUrl(uuid, '1')}, body: body1`,
       `${createBidderBeaconUrl(uuid, '2')}, body: body2`],
      // renderUrlOverride:
      createRenderUrl(
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

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      null,
      `registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid)}, body: body`,
       `${createBidderBeaconUrl(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller","buyer"]
          });`)
  );
}, 'Seller and buyer call registerAdBeacon() with shared reportEvent() call.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      null,
      `registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid)}, body: body1`,
       `${createBidderBeaconUrl(uuid)}, body: body2`],
      // renderUrlOverride:
      createRenderUrl(
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

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      // Multiple registerAdBeacon() call should result in an exception,
      // throwing away all beacons and other types of reports.
      `sendReportTo('${createSellerReportUrl(uuid)}');
       registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});
       registerAdBeacon({beacon1: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      'sellerSignals === null',
      `registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createBidderBeaconUrl(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller","buyer"]
          });`)
  );
}, 'Seller calls registerAdBeacon() multiple times.');

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
      test, uuid,
      // reportResult:
      null,
      `registerAdBeacon({beacon: '${createSellerBeaconUrl(uuid)}'});`,
      // reportWin:
      null,
      // Multiple registerAdBeacon() call should result in an exception,
      // throwing away all beacons and other types of reports.
      `sendReportTo('${createBidderReportUrl(uuid)}');
       registerAdBeacon({beacon: '${createBidderBeaconUrl(uuid)}'});
       registerAdBeacon({beacon1: '${createBidderBeaconUrl(uuid)}'});`,
      // expectedReportUrls:
      [`${createSellerBeaconUrl(uuid)}, body: body`],
      // renderUrlOverride:
      createRenderUrl(
          uuid,
          `window.fence.reportEvent({
              eventType: "beacon",
              eventData: "body",
              destination: ["seller","buyer"]
          });`)
  );
}, 'Buyer calls registerAdBeacon() multiple times.');
