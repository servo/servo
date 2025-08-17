const ws = require('ws')
const ws_server = new ws.Server({ host: '0.0.0.0', port: 8001 });
const getNow = () => {
  const date = new Date();
  return date.toLocaleTimeString() + "." + date.getMilliseconds();
};
console.log(getNow() + " WebSocket server started.");
const arrayBuf = 1000*1000; // 1MB
const totalIter = 100;

const charArray1000 = [];
for (i = 0; i < 1000; i++) {
  charArray1000.push(i % 128);
}
const asciiArray1K = String.fromCharCode.apply(this, charArray1000);
const textArray1M = [];
for (i = 0; i < 1000; i++) {
  textArray1M.push(asciiArray1K);
}
const asciiText1MB = textArray1M.join('');

ws_server.on('connection', function(ws_socket, request) {
  console.log(getNow() + ' Connection established. url=' + request.url);
  if (request.url === '/') {
    const data = new ArrayBuffer(arrayBuf);
    for (let i = 0; i < totalIter; i++) {
      ws_socket.send(data, {binary: true});
    }
  } else if (request.url === '/text') {
    for (let i = 0; i < totalIter; i++) {
      ws_socket.send(asciiText1MB);
    }
  } else {
    console.log('Invalid request: ' + request.url);
  }
  ws_socket.close();
  console.log(getNow() + " Connection closed.");
});