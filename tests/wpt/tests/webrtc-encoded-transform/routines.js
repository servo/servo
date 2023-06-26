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
