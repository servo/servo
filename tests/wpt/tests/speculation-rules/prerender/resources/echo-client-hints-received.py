""" Handle the initiator navigation request and attach the received client info
to the returned page.
"""


import textwrap

html_template = """
<!DOCTYPE html>
<html>
<head>
<title>echo client hints on prerendering page</title>
</head>
<script src="/speculation-rules/prerender/resources/utils.js"></script>
<body>
<script>

// Allow generator to add the received CH information into this script.
%s
const params = new URLSearchParams(location.search);
const uid = params.get('uid');

// Performs the check below on initiator pages:
// 1. The client did not send server_received_full_version_list when fetching
//    the initiator page.
// If the check fails, it will ask the main test page to terminate the test.
// Otherwise, it will:
// 1. Initiate a prerendering action. And the prerendering page will perform
//    some checks.
// 2. Wait for the prerendering page to pass all checks and send a signal back.
// 3. Activate the prerendered page.
async function load_as_initiator_page() {
  if (!server_received_bitness || server_received_full_version_list) {
    // The initial headers are not as expected. Terminate the test.
    failTest(
        `unexpected initial headers.
             bitness: ${server_received_bitness},
             full_version: ${server_received_full_version_list}`,
        uid);
    return;
  }
  const prerendering_url =
      `./echo-prerender-page-client-hints-received.py?uid=${uid}`;
  // Wait for the prerendered page to be ready for activation.
  const bc = new PrerenderChannel('prerender-channel', uid);
  const gotMessage = new Promise(resolve => {
    bc.addEventListener('message', e => {
      resolve(e.data);
    }, {once: true});
  });
  startPrerendering(prerendering_url);

  data = await gotMessage;
  if (data == 'ready for activation') {
    window.location = prerendering_url;
  } else {
    failTest(`Initial page received unexpected result: ${data}`, uid);
  }
}

load_as_initiator_page();

</script>
</body>
</html>
"""

def translate_to_js(val: bool) -> str:
    if isinstance(val, bool):
        return "true" if val else "false"
    return ""

def main(request, response):
    response.status = 200

    # Insert the received hints into script.
    content = html_template % (
        textwrap.dedent(
            f"""
            const server_received_bitness =
                {translate_to_js(b"sec-ch-ua-bitness" in request.headers)};
            const server_received_full_version_list =
                {translate_to_js(b"sec-ch-ua-full-version-list" in
                    request.headers)};
            """
        )
    )
    response.content = content.encode("utf-8")
