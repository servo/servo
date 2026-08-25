/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { globalTestConfig } from '../framework/test_config.js'; /**
 * - 'uninitialized' means we haven't tried to connect yet
 * - Promise means it's pending
 * - 'failed' means it failed (this is the most common case, where the logger isn't running)
 * - WebSocket means it succeeded
 */
let connection =
'uninitialized';

/**
 * If the logToWebSocket option is enabled (?log_to_web_socket=1 in browser,
 * --log-to-web-socket on command line, or enable it by default in options.ts),
 * log a string to a websocket at `localhost:59497`. See `tools/websocket-logger`.
 *
 * This does nothing if a logToWebSocket is not enabled, or if a connection
 * couldn't be established on the first call.
 */
export function logToWebSocket(msg) {
  if (!globalTestConfig.logToWebSocket || connection === 'failed') {
    return;
  }

  if (connection === 'uninitialized') {
    connection = new Promise((resolve) => {
      if (typeof WebSocket === 'undefined') {
        resolve('failed');
        return;
      }

      const ws = new WebSocket('ws://localhost:59497/optional_cts_websocket_logger');
      ws.onopen = () => {
        resolve(ws);
      };
      ws.onerror = () => {
        connection = 'failed';
        resolve('failed');
      };
      ws.onclose = () => {
        connection = 'failed';
        resolve('failed');
      };
    });
    void connection.then((resolved) => {
      connection = resolved;
    });
  }

  void (async () => {
    // connection may be a promise or a value here. Either is OK to await.
    const ws = await connection;
    if (ws !== 'failed') {
      ws.send(msg);
    }
  })();
}