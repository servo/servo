// META: title=Encrypted Media Extensions: Test handling of shared buffers
// META: script=/encrypted-media/util/utils.js

// None of the BufferSource arguments in Encrypted Media Extensions are
// [AllowShared], so a SharedArrayBuffer, or a view onto one, has to throw a
// TypeError.
//
// See https://github.com/whatwg/html/issues/5380 for why not `new SharedArrayBuffer()`.
const sharedBuffer = new WebAssembly.Memory({ shared: true, initial: 1, maximum: 1 }).buffer;

async function createMediaKeys() {
    const access = await navigator.requestMediaKeySystemAccess('org.w3.clearkey', getSimpleConfiguration());
    return access.createMediaKeys();
}

for (const [kind, buffer] of [['SharedArrayBuffer', sharedBuffer],
                              ['Uint8Array(SharedArrayBuffer)', new Uint8Array(sharedBuffer)]]) {
    // Converting the argument precedes the check for server certificate
    // support, which Clear Key does not have, so a user agent that wrongly
    // accepts the shared buffer resolves with false instead.
    promise_test(async t => {
        const mediaKeys = await createMediaKeys();
        await promise_rejects_js(t, TypeError, mediaKeys.setServerCertificate(buffer));
    }, `setServerCertificate() serverCertificate is a ${kind}`);

    promise_test(async t => {
        const keyStatuses = (await createMediaKeys()).createSession().keyStatuses;
        assert_throws_js(TypeError, () => keyStatuses.has(buffer));
    }, `MediaKeyStatusMap.has() keyId is a ${kind}`);

    promise_test(async t => {
        const keyStatuses = (await createMediaKeys()).createSession().keyStatuses;
        assert_throws_js(TypeError, () => keyStatuses.get(buffer));
    }, `MediaKeyStatusMap.get() keyId is a ${kind}`);
}
