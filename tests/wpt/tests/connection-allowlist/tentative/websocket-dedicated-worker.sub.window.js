// The following tests assume the policy Connection-Allowlist: (response-origin
// "http://{{host}}:{{ports[ws][0]}}") is set. The dedicated worker script will
// inherit the allowlisted policies from the document because it is loaded from
// a data: URL. It should then enforce its own allowlist on WebSocket
// connections based on that inheritance.

const ws_port = '{{ports[ws][0]}}';
const SUCCESS = true;
const FAILURE = false;

// The worker content will attempt to create a WebSocket and post the results
// back.
const worker_content = `
  onmessage = async (e) => {
    const url = e.data;
    try {
      const ws = new WebSocket(url);

      const result = await new Promise(resolve => {
        ws.onopen = () => { ws.close(); resolve(true); };
        ws.onerror = () => resolve(false);
      });
      postMessage({ url: url, success: result });
    } catch (err) {
      postMessage({ url: url, success: false, error: err.name });
    }
  };
`;
const dataUrl = 'data:text/javascript,' + encodeURIComponent(worker_content);

function dedicated_worker_websocket_test(host, expectation, description) {
  promise_test(async t => {
    const worker = new Worker(dataUrl, {type: 'module'});

    const ws_url = `ws://${host}:${ws_port}/echo`;
    worker.postMessage(ws_url);

    const msgEvent = await new Promise((resolve, reject) => {
      worker.onmessage = resolve;
      worker.onerror = (e) => reject(new Error('Worker Error'));
    });

    if (expectation === SUCCESS) {
      assert_true(
          msgEvent.data.success,
          `WebSocket connection to ${host} should succeed.`);
    } else {
      assert_false(
          msgEvent.data.success,
          `WebSocket connection to ${host} should be blocked.`);
    }
  }, description);
}

// 1. Same-origin WebSocket from the worker should succeed as it is allowlisted.
dedicated_worker_websocket_test(
    '{{hosts[][]}}', SUCCESS,
    'Same-origin WebSocket from a dedicated worker (data: URL) succeeds.');

// 2. Cross-origin WebSocket from the worker should be blocked as it is not
// allowlisted.
dedicated_worker_websocket_test(
    '{{hosts[alt][]}}', FAILURE,
    'Cross-origin WebSocket from a dedicated worker (data: URL) should be blocked by inherited policy.');
