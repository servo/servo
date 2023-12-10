/**
 * Create a new promise that resolves when the window receives
 * the MessagePort and starts it.
 *
 * @param {Window} window - The window to wait for the MessagePort.
 * @returns {Promise<MessagePort>} A promise you should await to ensure the
 *     window
 * receives the MessagePort.
 */
function expectMessagePortFromWindow(window) {
  return new Promise(resolve => {
    window.onmessage = e => {
      try {
        assert_true(e.ports[0] instanceof window.MessagePort);
        e.ports[0].start();
        resolve(e.ports[0]);
      } catch (e) {
        reject(e);
      }
    };
  });
}

/**
 * Create a new MessageChannel and transfers one of the ports to
 * the window which opened the window with a remote context provided
 * as an argument.
 *
 * @param {RemoteContextWrapper} remoteContextWrapper
 */
async function createMessageChannelAndSendPort(remoteContextWrapper) {
  await remoteContextWrapper.executeScript(() => {
    const {port1, port2} = new MessageChannel();
    port1.start();
    window.opener.postMessage({}, '*', [port2]);
    window.closePort = () => {
      port1.close();
    }
  });
}

/**
 * Creates a window with a remote context.
 *
 * @returns {Promise<RemoteContextWrapper>}
 */
async function addWindow() {
  const helper = new RemoteContextHelper();
  return helper.addWindow();
}

/**
 * Creates a new promise that resolves when the close event is fired.
 *
 * @param {MessagePort} port - MessagePort on which the close event will
 * be fired.
 * @returns {Promise} A promise you should await to ensure the close event
 * is dispatched.
 */
function createCloseEventPromise(port) {
  return new Promise(resolve => port.onclose = resolve);
}
