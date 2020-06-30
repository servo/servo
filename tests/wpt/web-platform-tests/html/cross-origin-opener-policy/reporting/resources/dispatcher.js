// Define an universal message passing API.
//
// In particular, this works:
// - cross-origin and
// - cross-browsing-context-group.
//
// It can also be used to receive reports.

const dispatcher_path =
    '/html/cross-origin-opener-policy/reporting/resources/dispatcher.py';
const dispatcher_url = new URL(dispatcher_path, location.href).href;

const send = function(uuid, message) {
  fetch(dispatcher_url + `?uuid=${uuid}`, {
    method: 'POST',
    body: message
  });
}

const receive = async function(uuid) {
  const timeout = 3000;
  const retry_delay = 100;
  for(let i = 0; i * retry_delay < timeout; ++i) {
    let response = await fetch(dispatcher_url + `?uuid=${uuid}`);
    let data = await response.text();
    if (data != 'not ready')
      return data;
    await new Promise(r => step_timeout(r, retry_delay));
  }
  return "timeout";
}

const receiveReport = async function(uuid, type) {
  while(true) {
    let reports = await receive(uuid);
    if (reports == "timeout")
      return "timeout";
    reports = JSON.parse(reports);

    for(report of reports) {
      if (report?.["body"]?.["violation-type"] == type)
        return report;
    }
  }
}

// Build a set of headers to tests the reporting API. This defines a set of
// matching 'Report-To', 'Cross-Origin-Opener-Policy' and
// 'Cross-Origin-Opener-Policy-Report-Only' headers.
const reportToHeaders = function(uuid) {
  const report_endpoint_url = dispatcher_path + `?uuid=${uuid}`;
  let reportToJSON = {
    'group': `${uuid}`,
    'max_age': 3600,
    'endpoints': [
      {'url': report_endpoint_url.toString()},
    ]
  };
  reportToJSON = JSON.stringify(reportToJSON)
                     .replace(/,/g, '\\,')
                     .replace(/\(/g, '\\\(')
                     .replace(/\)/g, '\\\)=');

  return {
    header: `|header(report-to,${reportToJSON})`,
    coopSameOriginHeader: `|header(Cross-Origin-Opener-Policy,same-origin%3Breport-to="${uuid}")`,
    coopReportOnlySameOriginHeader: `|header(Cross-Origin-Opener-Policy-Report-Only,same-origin%3Breport-to="${uuid}")`,
  };
};
