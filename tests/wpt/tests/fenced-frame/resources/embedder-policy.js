// This file should be loaded alongside with utils.js.
//
// This file is loaded by:
// - embedder-no-coep.https.html
// - embedder-require-corp.https.html

// Make input list to be used as a wptserve pipe
// (https://web-platform-tests.org/writing-tests/server-pipes.html).
// e.g.
// args: ['content-type,text/plain','Age,0']
// return: 'header(content-type,text/plain)|header(Age,0)'
function generateHeader(headers) {
  return headers.map((h) => {
    return 'header(' + h + ')';
  }).join('|');
}

// Setup a fenced frame for embedder-* WPTs.
async function setupTest(test_type, uuid, hostname='') {
  let headers = ["Supports-Loading-Mode,fenced-frame"];
  switch (test_type) {
    case "coep:require-corp":
      headers.push("cross-origin-embedder-policy,require-corp");
      headers.push("cross-origin-resource-policy,same-origin");
      break;
    case "no coep":
      break;
    default:
      assert_unreachable("unknown test_type:" + test_type);
      break;
  }
  const tmp_url = new URL('resources/embeddee.html', location.href);
  if (hostname) {
    tmp_url.hostname = hostname;
  }
  tmp_url.searchParams.append("pipe", generateHeader(headers));
  const url = generateURL(tmp_url.toString(), [uuid]);
  return attachFencedFrame(url);
}
