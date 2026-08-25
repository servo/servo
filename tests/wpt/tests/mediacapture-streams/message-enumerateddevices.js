onmessage = async e => {
  const stream = await navigator.mediaDevices.getUserMedia({audio: true, video: true});
  stream.getTracks().forEach(t => t.stop());
  const devices = await navigator.mediaDevices.enumerateDevices();
  e.source.postMessage({
    devices: devices.map(d => d.toJSON())
  }, '*');
}
