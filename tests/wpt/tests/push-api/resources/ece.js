async function HKDF({ salt, ikm, info, length }) {
  return await crypto.subtle.deriveBits(
    { name: "HKDF", hash: "SHA-256", salt, info },
    await crypto.subtle.importKey("raw", ikm, { name: "HKDF" }, false, [
      "deriveBits",
    ]),
    length * 8
  );
}

// https://datatracker.ietf.org/doc/html/rfc8188#section-2.2
// https://datatracker.ietf.org/doc/html/rfc8188#section-2.3
async function deriveKeyAndNonce(header) {
  const { salt } = header;
  const ikm = await getInputKeyingMaterial(header);

  // cek_info = "Content-Encoding: aes128gcm" || 0x00
  const cekInfo = new TextEncoder().encode("Content-Encoding: aes128gcm\0");
  // nonce_info = "Content-Encoding: nonce" || 0x00
  const nonceInfo = new TextEncoder().encode("Content-Encoding: nonce\0");

  // (The XOR SEQ is skipped as we only create single record here, thus becoming noop)
  return {
    // the length (L) parameter to HKDF is 16
    key: await HKDF({ salt, ikm, info: cekInfo, length: 16 }),
    // The length (L) parameter is 12 octets
    nonce: await HKDF({ salt, ikm, info: nonceInfo, length: 12 }),
  };
}

// https://datatracker.ietf.org/doc/html/rfc8291#section-3.3
// https://datatracker.ietf.org/doc/html/rfc8291#section-3.4
async function getInputKeyingMaterial(header) {
  // IKM:  the shared secret derived using ECDH
  // ecdh_secret = ECDH(as_private, ua_public)
  const ikm = await crypto.subtle.deriveBits(
    {
      name: "ECDH",
      public: await crypto.subtle.importKey(
        "raw",
        header.userAgentPublicKey,
        { name: "ECDH", namedCurve: "P-256" },
        true,
        []
      ),
    },
    header.appServer.privateKey,
    256
  );
  // key_info = "WebPush: info" || 0x00 || ua_public || as_public
  const keyInfo = new Uint8Array([
    ...new TextEncoder().encode("WebPush: info\0"),
    ...header.userAgentPublicKey,
    ...header.appServer.publicKey,
  ])
  return await HKDF({ salt: header.authSecret, ikm, info: keyInfo, length: 32 });
}

// https://datatracker.ietf.org/doc/html/rfc8188#section-2
async function encryptRecord(key, nonce, data) {
  // add a delimiter octet (0x01 or 0x02)
  // The last record uses a padding delimiter octet set to the value 2
  //
  // (This implementation only creates a single record, thus always 2,
  // per https://datatracker.ietf.org/doc/html/rfc8291/#section-4:
  // An application server MUST encrypt a push message with a single
  // record.)
  const padded = new Uint8Array([...data, 2]);

  // encrypt with AEAD_AES_128_GCM
  return await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: nonce, tagLength: 128 },
    await crypto.subtle.importKey("raw", key, { name: "AES-GCM" }, false, [
      "encrypt",
    ]),
    padded
  );
}

// https://datatracker.ietf.org/doc/html/rfc8188#section-2.1
function writeHeader(header) {
  var dataView = new DataView(new ArrayBuffer(5));
  dataView.setUint32(0, header.recordSize);
  dataView.setUint8(4, header.keyid.length);
  return new Uint8Array([
    ...header.salt,
    ...new Uint8Array(dataView.buffer),
    ...header.keyid,
  ]);
}

function validateParams(params) {
  const header = { ...params };
  if (!header.salt) {
    throw new Error("Must include a salt parameter");
  }
  if (header.salt.length !== 16) {
    // https://datatracker.ietf.org/doc/html/rfc8188#section-2.1
    // The "salt" parameter comprises the first 16 octets of the
    // "aes128gcm" content-coding header.
    throw new Error("The salt parameter must be 16 bytes");
  }
  if (header.appServer.publicKey.byteLength !== 65) {
    // https://datatracker.ietf.org/doc/html/rfc8291#section-4
    // A push message MUST include the application server ECDH public key in
    // the "keyid" parameter of the encrypted content coding header.  The
    // uncompressed point form defined in [X9.62] (that is, a 65-octet
    // sequence that starts with a 0x04 octet) forms the entirety of the
    // "keyid".
    throw new Error("The appServer.publicKey parameter must be 65 bytes");
  }
  if (!header.authSecret) {
    throw new Error("No authentication secret for webpush");
  }
  return header;
}

export async function encrypt(data, params) {
  const header = validateParams(params);

  // https://datatracker.ietf.org/doc/html/rfc8291#section-2
  // The ECDH public key is encoded into the "keyid" parameter of the encrypted content coding header
  header.keyid = header.appServer.publicKey;
  header.recordSize = data.byteLength + 18 + 1;

  // https://datatracker.ietf.org/doc/html/rfc8188#section-2
  // The final encoding consists of a header (see Section 2.1) and zero or more
  // fixed-size encrypted records; the final record can be smaller than the record size.
  const saltedHeader = writeHeader(header);
  const { key, nonce } = await deriveKeyAndNonce(header);
  const encrypt = await encryptRecord(key, nonce, data);
  return new Uint8Array([...saltedHeader, ...new Uint8Array(encrypt)]);
}
