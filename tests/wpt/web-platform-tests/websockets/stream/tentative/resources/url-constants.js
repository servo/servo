// The file including this must also include ../websocket.sub.js to pick up the
// necessary constants.

const {BASEURL, ECHOURL} = (() => {
  const isSecure = location.href.match(/^https:/);
  const scheme = isSecure ? "wss:" : "ws:";
  const port = isSecure ? __SECURE__PORT : __PORT;
  const BASEURL = `${scheme}//${__SERVER__NAME}:${port}`;
  const ECHOURL = `${BASEURL}/echo`;
  return {BASEURL, ECHOURL};
})();
