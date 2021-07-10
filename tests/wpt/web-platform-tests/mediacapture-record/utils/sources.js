function createAudioStream(t) {
  const ac = new AudioContext();
  const { stream } = ac.createMediaStreamDestination();
  const [track] = stream.getTracks();
  t.add_cleanup(() => {
    ac.close();
    track.stop();
  });
  return { stream };
}

function createFlowingAudioStream(t) {
  const ac = new AudioContext();
  const dest = ac.createMediaStreamDestination();
  const osc = ac.createOscillator();
  osc.connect(dest);
  osc.start();
  const [track] = dest.stream.getTracks();
  t.add_cleanup(() => {
    ac.close();
    track.stop();
  });
  return { stream: dest.stream };
}

function createVideoStream(t) {
  const canvas = document.createElement("canvas");
  canvas.id = "canvas";
  document.body.appendChild(canvas);
  const ctx = canvas.getContext("2d");
  const stream = canvas.captureStream();
  const [track] = stream.getTracks();
  t.add_cleanup(() => {
    document.body.removeChild(canvas);
    track.stop();
  });
  const addVideoFrame = () => {
    ctx.fillStyle = "red";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  };
  return { stream, control: { addVideoFrame } };
}

function createFlowingVideoStream(t) {
  const { stream } = createVideoStream(t);
  const [track] = stream.getTracks();
  const canvas = document.getElementById("canvas");
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "green";
  requestAnimationFrame(function draw() {
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    if (track.readyState == "live") {
      requestAnimationFrame(draw);
    }
  });
  return { stream };
}

function createAudioVideoStream(t) {
  const { stream: audio } = createAudioStream(t);
  const { stream: video, control } = createVideoStream(t);
  return {
    stream: new MediaStream([...audio.getTracks(), ...video.getTracks()]),
    control,
  };
}

function createFlowingAudioVideoStream(t) {
  return {
    stream: new MediaStream([
      ...createFlowingAudioStream(t).stream.getTracks(),
      ...createFlowingVideoStream(t).stream.getTracks(),
    ]),
  };
}
