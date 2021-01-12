Why are there `.headers` files here for the `.mjs` scripts?

Because `../getter-special-cases/sandboxed-iframe.sub.https.html` is testing an
opaque origin, which is cross-origin with these scripts. Since
`<script type="module">` respects the same-origin policy, we need CORS headers
to allow them to be accessed.
