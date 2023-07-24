onmessage = async e => {
  const devices = await navigator.mediaDevices.enumerateDevices();
  e.source.postMessage({
    devices: devices.map(d => d.toJSON())
  }, '*');
}
