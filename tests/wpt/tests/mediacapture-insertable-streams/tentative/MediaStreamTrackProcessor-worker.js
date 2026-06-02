onmessage = async msg => {
  const reader = msg.data.readable.getReader();
  let readResult = await reader.read();
  postMessage(readResult.value);
  readResult.value.close();
  // Continue reading until the stream is done due to a track.stop()
  while (true) {
    readResult = await reader.read();
    if (readResult.done) {
      break;
    } else {
      readResult.value.close();
    }
  }
  await reader.closed;
  postMessage('closed');
}
