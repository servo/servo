const STASH_RESPONDER = "wss://{{host}}:{{ports[wss][0]}}/stash_responder_blocking";

class StashUtils {
  /**
   * Sends a request to store (|key|, |tuple|) in Stash
   * (https://web-platform-tests.org/tools/wptserve/docs/stash.html).
   * @param {string} key A UUID that acts as a key that can be used to retrieve |value| later.
   * @param {string} value Value to be stored in Stash.
   * @returns {Promise} Promise that resolves once the server responds.
   */
  static putValue(key, value) {
    return new Promise(resolve => {
        const ws = new WebSocket(STASH_RESPONDER);
        ws.onopen = () => {
          ws.send(JSON.stringify({action: 'set', key: key, value: value}));
        };
        ws.onmessage = e => {
          ws.close();
          resolve();
        };
    });
  }

  /**
   * Retrieves value associated with |key| in Stash. If no value has been
   * associated with |key| yet, the method waits for putValue to be called with
   * |key|, and a value to be associated, before resolving the return promise.
   * @param {string} key A UUID that uniquely identifies the value to retrieve.
   * @returns {Promise<string>} A promise that resolves with the value associated with |key|.
   */
  static takeValue(key) {
    return new Promise(resolve => {
      const ws = new WebSocket(STASH_RESPONDER);
      ws.onopen = () => {
        ws.send(JSON.stringify({action: 'get', key: key}));
      };
      ws.onmessage = e => {
        ws.close();
        resolve(JSON.parse(e.data).value);
      };
    });
  }
}
