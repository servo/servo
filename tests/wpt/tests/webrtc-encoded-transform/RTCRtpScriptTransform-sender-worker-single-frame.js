onrtctransform = async (event) => {
  const readableStream = event.transformer.readable;
  const reader = readableStream.getReader();
  const result = await reader.read();
  // Post an object with individual fields so that the test side has
  // values to verify the serialization of the RTCEncodedVideoFrame.
  postMessage({
    type: result.value.type,
    data: result.value.data,
    metadata: result.value.getMetadata(),
  });
  // Send the frame twice to verify that the frame does not change after the
  // first serialization.
  postMessage(result.value);
  postMessage(result.value);
}