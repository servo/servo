let port;
onmessage = async e => {
    await self.clients.claim();

    port = e.data.port;
    port.postMessage("Ready");
}

onfetch = e => {
    const request = e.request;
    if (port && request.url.endsWith(".jpg")) {
       port.postMessage({ type: "fetch", info: {
            url : request.url,
            destination : request.destination,
            mode : request.mode,
            redirect : request.redirect,
            referrerPolicy : request.referrerPolicy,
            credentials : request.credentials
       }});
    }
}
