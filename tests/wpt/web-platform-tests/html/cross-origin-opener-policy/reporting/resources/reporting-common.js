
const directory = "/html/cross-origin-opener-policy/reporting/resources";
const executor_path = directory + "/executor.html?pipe=";
const coep_header = '|header(Cross-Origin-Embedder-Policy,require-corp)';

// Allows RegExps to be pretty printed when printing unmatched expected reports.
Object.defineProperty(RegExp.prototype, "toJSON", {
  value: RegExp.prototype.toString
});

function wait(ms) {
  return new Promise(resolve => step_timeout(resolve, ms));
}

// Check whether a |report| is a "opener breakage" COOP report.
function isCoopOpenerBreakageReport(report) {
  if (report.type != "coop")
    return false;

  if (report.body["violation-type"] != "navigation-from-document" &&
      report.body["violation-type"] != "navigation-to-document") {
    return false;
  }

  return true;
}

async function pollReports(endpoint) {
  const res = await fetch(
    `${directory}/report.py?endpoint=${endpoint.name}`,
      {cache: 'no-store'});
  if (res.status !== 200) {
    return;
  }
  for (const report of await res.json()) {
    if (isCoopOpenerBreakageReport(report))
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
// EXECUTOR_UUID: the uuid generated with token().
function replaceValuesInExpectedReport(expectedReport, executorUuid) {
  if (expectedReport.report.body !== undefined) {
    if (expectedReport.report.body["document-uri"] !== undefined) {
      expectedReport.report.body["document-uri"] = replaceFromRegexOrString(
          expectedReport.report.body["document-uri"], "EXECUTOR_UUID",
          executorUuid);
    }
    if (expectedReport.report.body["navigation-uri"] !== undefined) {
      expectedReport.report.body["navigation-uri"] = replaceFromRegexOrString(
          expectedReport.report.body["navigation-uri"], "EXECUTOR_UUID",
          executorUuid);
    }
  }
  if (expectedReport.report.url !== undefined) {
      expectedReport.report.url = replaceFromRegexOrString(
          expectedReport.report.url, "EXECUTOR_UUID", executorUuid);
  }
  return expectedReport;
}

// Run a test (such as coop_coep_test from ./common.js) then check that all
// expected reports are present.
async function reportingTest(testFunction, executorToken, expectedReports) {
  await new Promise(testFunction);
  expectedReports = Array.from(
      expectedReports,
      report => replaceValuesInExpectedReport(report, executorToken) );
  await Promise.all(Array.from(expectedReports, checkForExpectedReport));
}

function getReportEndpoints(host) {
  result = "";
  reportEndpoints.forEach(
    reportEndpoint => {
      let reportToJSON = {
        'group': `${reportEndpoint.name}`,
        'max_age': 3600,
        'endpoints': [
          {'url': `${host}/html/cross-origin-opener-policy/reporting/resources/report.py?endpoint=${reportEndpoint.name}`
          },
        ]
      };
      result += JSON.stringify(reportToJSON)
                        .replace(/,/g, '\\,')
                        .replace(/\(/g, '\\\(')
                        .replace(/\)/g, '\\\)=')
                + "\\,";
    }
  );
  return result.slice(0, -2);
}

function navigationReportingTest(testName, host, coop, coep, coopRo, coepRo,
    expectedReports ){
  const executorToken = token();
  const callbackToken = token();
  promise_test(async t => {
    await reportingTest( async resolve => {
      const openee_url = host.origin + executor_path +
      `|header(report-to,${encodeURIComponent(getReportEndpoints(host.origin))})` +
      `|header(Cross-Origin-Opener-Policy,${encodeURIComponent(coop)})` +
      `|header(Cross-Origin-Embedder-Policy,${encodeURIComponent(coep)})` +
      `|header(Cross-Origin-Opener-Policy-Report-Only,${encodeURIComponent(coopRo)})` +
      `|header(Cross-Origin-Embedder-Policy-Report-Only,${encodeURIComponent(coepRo)})`+
      `&uuid=${executorToken}`;
      const openee = window.open(openee_url);
      t.add_cleanup(() => send(5, "window.close()"));

      // 1. Make sure the new document is loaded.
      send(executorToken, `
        send("${callbackToken}", "Ready");
      `);
      let reply = await receive(callbackToken);
      assert_equals(reply, "Ready");
      resolve();
    }, executorToken, expectedReports);
  }, `coop reporting test ${testName} to ${host.name} with ${coop}, ${coep}, ${coopRo}, ${coepRo}`);
}

// Run an array of reporting tests then verify there's no reports that were not
// expected.
// Tests' elements contain: host, coop, coep, hasOpener, expectedReports.
// See isObjectAsExpected for explanations regarding the matching behavior.
function runNavigationReportingTests(testName, tests){
  tests.forEach( test => {
    navigationReportingTest(testName, ...test);
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
