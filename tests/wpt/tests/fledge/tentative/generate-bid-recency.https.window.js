// META: script=/resources/testdriver.js
// META: script=/common/utils.js
// META: script=resources/fledge-util.sub.js
// META: timeout=long

"use strict;"

promise_test(async test => {
  const uuid = generateUuid(test);
  await runReportTest(
    test, uuid,
    { generateBid:
        `if (browserSignals.recency === undefined)
           throw new Error("Missing recency in browserSignals.")

         if (browserSignals.recency < 0)
           throw new Error("Recency is a negative value.")

         if (browserSignals.recency > 30000)
           throw new Error("Recency is over 30 seconds threshold.")

         if (browserSignals.recency % 100 !== 0)
           throw new Error("Recency is not rounded to multiple of 100 milliseconds.")

         return {'bid': 9,
                 'render': interestGroup.ads[0].renderURL};`,
      reportWin:
        `sendReportTo('${createBidderReportURL(uuid)}');`
    },
    // expectedReportUrls
    [createBidderReportURL(uuid)]
  );
}, 'Check recency in generateBid() is below a certain threshold and rounded ' +
   'to multiple of 100 milliseconds.');
