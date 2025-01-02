async function getNextMessage(portOrWorker) {
  return new Promise(resolve => {
    const resolveWithData = event => resolve(event.data);
    const rejectWithData = event => reject(event.data);
    portOrWorker.addEventListener('message', resolveWithData, {once: true});
    portOrWorker.addEventListener('messageerror', rejectWithData, {once: true});
  });
}


async function postMethod(port, method, options) {
  port.postMessage(Object.assign({method}, options));
  return await getNextMessage(port);
}

async function createWorker(script) {
  const worker = new Worker(script);
  const data = await getNextMessage(worker);
  assert_equals(data, "registered");
  return worker;
}

async function createTransform(worker) {
  const channel = new MessageChannel;
  const transform = new RTCRtpScriptTransform(worker, {name:'MockRTCRtpTransform', port: channel.port2}, [channel.port2]);
  transform.port = channel.port1;
  channel.port1.start();
  assert_equals(await getNextMessage(channel.port1), "started");
  return transform;
}

async function createTransforms(script) {
  const worker = await createWorker(script)
  return Promise.all([createTransform(worker), createTransform(worker)]);
}

async function createConnectionWithTransform(test, script, gumOptions) {
  const [senderTransform, receiverTransform] = await createTransforms(script);

  const localStream = await navigator.mediaDevices.getUserMedia(gumOptions);

  let senderPc, receiverPc, sender, receiver;

  await createConnections(test, (firstConnection) => {
      senderPc = firstConnection;
      sender = firstConnection.addTrack(localStream.getTracks()[0], localStream);
      sender.transform = senderTransform;
    }, (secondConnection) => {
      receiverPc = secondConnection;
      secondConnection.ontrack = (trackEvent) => {
        receiver = trackEvent.receiver;
        receiver.transform = receiverTransform;
      };
    });

  assert_true(!!sender, "sender should be set");
  assert_true(!!receiver, "receiver should be set");

  return {sender, receiver, senderPc, receiverPc};
}

async function createConnections(test, setupLocalConnection, setupRemoteConnection, doNotCloseAutmoatically) {
    const localConnection = new RTCPeerConnection();
    const remoteConnection = new RTCPeerConnection();

    remoteConnection.onicecandidate = (event) => { localConnection.addIceCandidate(event.candidate); };
    localConnection.onicecandidate = (event) => { remoteConnection.addIceCandidate(event.candidate); };

    await setupLocalConnection(localConnection);
    await setupRemoteConnection(remoteConnection);

    const offer = await localConnection.createOffer();
    await localConnection.setLocalDescription(offer);
    await remoteConnection.setRemoteDescription(offer);

    const answer = await remoteConnection.createAnswer();
    await remoteConnection.setLocalDescription(answer);
    await localConnection.setRemoteDescription(answer);

    if (!doNotCloseAutmoatically) {
        test.add_cleanup(() => {
            localConnection.close();
            remoteConnection.close();
        });
    }

    return [localConnection, remoteConnection];
}

function waitFor(test, duration)
{
    return new Promise((resolve) => test.step_timeout(resolve, duration));
}
