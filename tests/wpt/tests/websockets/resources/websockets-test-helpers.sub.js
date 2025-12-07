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

// Opens a new WebSocket connection and close it in pagehide event listener.
async function openWebSocketAndCloseItInPageHide(remoteContextHelper) {
  window.wsErrorOccurred = false;
  window.wsCloseOccurred = false;

  let return_value = await remoteContextHelper.executeScript((domain) => {
    return new Promise((resolve) => {
      var testWebSocket = new WebSocket(domain + '/echo');
      testWebSocket.onopen = () => {
        // Close WebSocket during pagehide (BFCache entry)
        window.addEventListener(
          'pagehide',
          () => testWebSocket.close()
        );
        resolve(42);
      };
      testWebSocket.onerror = () => { window.wsErrorOccurred = true; };
      testWebSocket.onclose = () => { window.wsCloseOccurred = true; };
    });
  }, [SCHEME_DOMAIN_PORT]);
  assert_equals(return_value, 42);
}

// Reads wsErrorOccurred and wsCloseOccurred from the remote context.
async function readWebSocketCloseAndErrorFlags(remoteContext) {
  return await remoteContext.executeScript(() => {
    return {
      wsError: window.wsErrorOccurred === true,
      wsClose: window.wsCloseOccurred === true
    };
  });
}
