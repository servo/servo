self.addEventListener("fetch", async e => {
    if (e.request.url.endsWith("ping"))
        e.respondWith(new Response('pong'));
    else if (e.request.url.endsWith("client")) {
        e.respondWith((async () => {
            const client = await clients.get(e.clientId);
            const clientInfo = client ? {id: e.clientId, visibilityState: client.visibilityState, focused: client.focused} : null;
            return new Response(JSON.stringify({clientInfo}), {headers: {'Content-Type': 'application/json'}});
        })());
    }
});
