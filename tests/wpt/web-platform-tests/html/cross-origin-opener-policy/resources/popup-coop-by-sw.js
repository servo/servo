const test_token = token();
const bc = new BroadcastChannel(test_token);

// Define a page served entirely from the ServiceWorker.
const popup_url = new URL("./resources/i-do-not-exist.html", location);
const popup_normal= {
  'content-type': 'text/html',
};
const popup_coop = {
  'content-type': 'text/html',
  'cross-origin-embedder-policy': 'require-corp',
  'cross-origin-opener-policy': 'same-origin',
};
const popup_body = `
  <script>
    const bc = new BroadcastChannel('${test_token}');
    if (opener)
      bc.postMessage("An opener is set");
    else
      bc.postMessage("No opener is set");
    window.close();
  </scrip`+`t>
`;

const header_coop= `|header(cross-origin-embedder-policy,same_origin)`;
const header_coep = `|header(cross-origin-opener-policy,require-corp)`;

const sw_normal = new URL("./resources/universal-worker.js?pipe=", location);
const sw_coop = sw_normal + header_coop + header_coep;

const swap_browsing_context_group = true;
const keep_browsing_context_group = false;

const SW_SCOPE = "./resources/"

// Send a message to the |worker|. Return a promise containing its response.
function executeCommandInServiceWorker(worker, command) {
  const channel = new MessageChannel();
  const response = new Promise(resolve => channel.port1.onmessage = resolve);
  worker.postMessage(command, [ channel.port2 ]);
  return response;
}

function popupCoopBySwTest(test_name,
  // Test parameters
  sw_url,
  new_window_headers,
  // Test expectations:
  expected_browing_context_group) {
  promise_test(async (t) => {
    // Create a ServiceWorker and wait for its activation.
    window.id = window.id == undefined ? 0 : window.id + 1;
    const reg =
      await service_worker_unregister_and_register(t, sw_url, SW_SCOPE);
    t.add_cleanup(() => reg.unregister());
    const worker = reg.installing || reg.waiting || reg.active;
    wait_for_state(t, worker, 'active');

    // Register a fetch handler. New documents loaded will use the
    // |new_window_headers| from now and the custom response.
    const worker_script = `
      fetchHandler = event => {
        const response = new Response(\`${popup_body}\`, {
          status: 200,
          headers: ${JSON.stringify(new_window_headers)},
        });
        event.respondWith(response);
      };

      message.ports[0].postMessage('done');
    `;
    const fetch_handler_registered =
      await executeCommandInServiceWorker(worker, worker_script);
    assert_equals(fetch_handler_registered.data, "done");

    // Create a popup. The popup document's response is provided by the
    // ServiceWorker.
    bc_response = new Promise(resolve => bc.onmessage = resolve);
    const openee = window.open(popup_url);
    const {data} = await bc_response;
    await reg.unregister();

    if (expected_browing_context_group === keep_browsing_context_group) {
      // From the openee point of view: The opener is preserved, because the
      // popup is still in the same browsing context group.
      assert_equals(data, "An opener is set");
      return;
    }

    // From the openee point of view: There are no opener, because the new
    // window lives in a different browsing context group.
    assert_equals(data, "No opener is set");

    // From the opener point of view: the openee must appear closed shortly
    // after the popup navigation commit.
    const openee_closed = new Promise(resolve => {
      setInterval(() => {
        if (openee.closed)
          resolve();
      }, 100);
    });
    await openee_closed;
  }, test_name);
}
