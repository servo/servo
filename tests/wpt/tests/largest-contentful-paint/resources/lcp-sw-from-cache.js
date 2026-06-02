self.addEventListener("fetch", e => {
    if (e.request.url.endsWith('green.svg')) {
        e.respondWith(new Response(`<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
        <rect fill="lime" width="100" height="50"/>
        </svg>
        `, { headers: { 'Content-Type': 'image/svg+xml' } }));
    }
});
