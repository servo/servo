// Allows RegExps to be pretty printed when printing unmatched expected reports.
Object.defineProperty(RegExp.prototype, "toJSON", {
  value: RegExp.prototype.toString
});

function wait(ms) {
  return new Promise(resolve => step_timeout(resolve, ms));
}

async function pollReports(endpoint) {
  const res = await fetch(
      `resources/report.py?endpoint=${endpoint.name}`,
      {cache: 'no-store'});
  if (res.status !== 200) {
    return;
  }
  for (const report of await res.json()) {
    endpoint.reports.push(report);
  }
}

// Recursively check that all members of expectedReport are present or matched
// in report.
// Report may have members not explicitly expected by expectedReport.
function isObjectAsExpected(report, expectedReport) {
  if (( report === undefined || report === null
        || expectedReport === undefined || expectedReport === null )
      && report !== expectedReport ) {
    return false;
  }
  if (expectedReport instanceof RegExp && typeof report === "string") {
    return expectedReport.test(report);
  }
  // Perform this check now, as RegExp and strings above have different typeof.
  if (typeof report !== typeof expectedReport)
    return false;
  if (typeof expectedReport === 'object') {
    return Object.keys(expectedReport).every(key => {
      return isObjectAsExpected(report[key], expectedReport[key]);
    });
  }
  return report == expectedReport;
}

async function checkForExpectedReport(expectedReport) {
  return new Promise( async (resolve, reject) => {
    const polls = 30;
    const waitTime = 100;
    for (var i=0; i < polls; ++i) {
      pollReports(expectedReport.endpoint);
      for (var j=0; j<expectedReport.endpoint.reports.length; ++j){
        if (isObjectAsExpected(expectedReport.endpoint.reports[j],
            expectedReport.report)){
          expectedReport.endpoint.reports.splice(j,1);
          resolve();
        }
      };
      await wait(waitTime);
    }
    reject("No report matched the expected report for endpoint: "
      + expectedReport.endpoint.name
      + ", expected report: " + JSON.stringify(expectedReport.report)
      + ", within available reports: "
      + JSON.stringify(expectedReport.endpoint.reports)
    );
  });
}

function replaceFromRegexOrString(str, match, value) {
  if (str instanceof RegExp) {
    return RegExp(str.source.replace(match, value));
  }
  return str.replace(match, value);
}

// Replace generated values in regexes and strings of an expected report:
// CHANNEL_NAME: the channel name is generated from the test parameters.
function replaceValuesInExpectedReport(expectedReport, channelName) {
  if (expectedReport.report.body !== undefined) {
    if (expectedReport.report.body["document-uri"] !== undefined) {
      expectedReport.report.body["document-uri"] = replaceFromRegexOrString(
          expectedReport.report.body["document-uri"], "CHANNEL_NAME",
          channelName);
    }
    if (expectedReport.report.body["navigation-uri"] !== undefined) {
      expectedReport.report.body["navigation-uri"] = replaceFromRegexOrString(
          expectedReport.report.body["navigation-uri"], "CHANNEL_NAME",
          channelName);
    }
  }
  if (expectedReport.report.url !== undefined) {
      expectedReport.report.url = replaceFromRegexOrString(
          expectedReport.report.url, "CHANNEL_NAME", channelName);
  }
  return expectedReport;
}

// Run a test (such as coop_coep_test from ./common.js) then check that all
// expected reports are present.
async function reportingTest(testFunction, channelName, expectedReports) {
  await new Promise( async resolve => {
    testFunction(resolve);
  });
  expectedReports = Array.from(
      expectedReports,
      report => replaceValuesInExpectedReport(report, channelName) );
  await Promise.all(Array.from(expectedReports, checkForExpectedReport));
}

function coopCoepReportingTest(testName, host, coop, coep, hasOpener,
    expectedReports){
  const channelName = `${testName.replace(/[ ;"=]/g,"-")}_to_${host.name}_${coop.replace(/[ ;"=]/g,"-")}_${coep}`;
  promise_test(async t => {
    await reportingTest( (resolve) => {
      coop_coep_test(t, host, coop, coep, channelName,
          hasOpener, undefined /* openerDOMAccess */, resolve);
    }, channelName, expectedReports);
  }, `coop reporting test ${channelName}`);
}

// Run an array of reporting tests then verify there's no reports that were not
// expected.
// Tests' elements contain: host, coop, coep, hasOpener, expectedReports.
// See isObjectAsExpected for explanations regarding the matching behavior.
function runCoopReportingTest(testName, tests){
  tests.forEach( test => {
    coopCoepReportingTest(testName, ...test);
  });
  verifyRemainingReports();
}

const reportEndpoint = {
  name: "coop-report-endpoint",
  reports: []
}
const reportOnlyEndpoint = {
  name: "coop-report-only-endpoint",
  reports: []
}
const popupReportEndpoint = {
  name: "coop-popup-report-endpoint",
  reports: []
}
const popupReportOnlyEndpoint = {
  name: "coop-popup-report-only-endpoint",
  reports: []
}
const redirectReportEndpoint = {
  name: "coop-redirect-report-endpoint",
  reports: []
}
const redirectReportOnlyEndpoint = {
  name: "coop-redirect-report-only-endpoint",
  reports: []
}

const reportEndpoints = [
  reportEndpoint,
  reportOnlyEndpoint,
  popupReportEndpoint,
  popupReportOnlyEndpoint,
  redirectReportEndpoint,
  redirectReportOnlyEndpoint
]

function verifyRemainingReports() {
  promise_test( async t => {
    await Promise.all(Array.from(reportEndpoints, (endpoint) => {
      return new Promise( async (resolve, reject) => {
        await pollReports(endpoint);
        if (endpoint.reports.length != 0)
          reject( `${endpoint.name} not empty`);
        resolve();
      });
    }));
  }, "verify remaining reports");
}
