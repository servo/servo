// The file including this must also include `/websockets/constants.sub.js to
// pick up the necessary constants.

// Opens a new WebSocket connection.
async function openWebSocket(remoteContextHelper) {
  let return_value = await remoteContextHelper.executeScript((domain) => {
    return new Promise((resolve) => {
      var webSocketInNotRestoredReasonsTests = new WebSocket(domain + '/echo');
      webSocketInNotRestoredReasonsTests.onopen = () => { resolve(42); };
    });
  }, [SCHEME_DOMAIN_PORT]);
  assert_equals(return_value, 42);
}

// Opens a new WebSocket connection and then close it.
async function openThenCloseWebSocket(remoteContextHelper) {
  let return_value = await remoteContextHelper.executeScript((domain) => {
    return new Promise((resolve) => {
      var testWebSocket = new WebSocket(domain + '/echo');
      testWebSocket.onopen = () => { testWebSocket.close() };
      testWebSocket.onclose = () => { resolve(42) };
    });
  }, [SCHEME_DOMAIN_PORT]);
  assert_equals(return_value, 42);
}
