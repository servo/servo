import { encrypt as eceEncrypt } from "./ece.js"

export async function encrypt(data, p256dhKey, authKey) {
  if (!(data instanceof Uint8Array)) {
    throw new Error("Expecting Uint8Array for `data` parameter");
  }

  const salt = crypto.getRandomValues(new Uint8Array(16));

  const keyPair = await crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ["deriveBits"]);
  const publicKey = new Uint8Array(await crypto.subtle.exportKey("raw", keyPair.publicKey));

  const body = await eceEncrypt(data, {
    userAgentPublicKey: new Uint8Array(p256dhKey),
    appServer: {
      privateKey: keyPair.privateKey,
      publicKey,
    },
    salt,
    authSecret: authKey,
  });

  const headers = {
    // https://datatracker.ietf.org/doc/html/rfc8291#section-4
    // The Content-Encoding header field therefore has exactly one value, which is "aes128gcm".
    'Content-Encoding': "aes128gcm",
    // https://datatracker.ietf.org/doc/html/rfc8030#section-5.2
    // An application server MUST include the TTL (Time-To-Live) header
    // field in its request for push message delivery.  The TTL header field
    // contains a value in seconds that suggests how long a push message is
    // retained by the push service.
    TTL: 15,
  };

  return {
    body,
    headers,
  }
}
