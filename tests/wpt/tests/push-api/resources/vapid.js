function toBase64Url(array) {
  return btoa([...array].map(c => String.fromCharCode(c)).join('')).replaceAll("+", "-").replaceAll("/", "_").replaceAll("=", "")
}

class VAPID {
  #publicKey;
  #privateKey;

  constructor(publicKey, privateKey) {
    this.#publicKey = publicKey;
    this.#privateKey = privateKey;
  }

  get publicKey() {
    return this.#publicKey;
  }

  async #jws(audience) {
    // https://datatracker.ietf.org/doc/html/rfc7515#section-3.1
    // BASE64URL(UTF8(JWS Protected Header)) || '.' ||
    // BASE64URL(JWS Payload) || '.' ||
    // BASE64URL(JWS Signature)

    // https://datatracker.ietf.org/doc/html/rfc8292#section-2
    // ECDSA on the NIST P-256 curve [FIPS186], which is identified as "ES256" [RFC7518].
    const rawHeader = { typ: "JWT", alg: "ES256" };
    const header = toBase64Url(new TextEncoder().encode(JSON.stringify(rawHeader)));

    // https://datatracker.ietf.org/doc/html/rfc8292#section-2
    const rawPayload = {
      // An "aud" (Audience) claim in the token MUST include the Unicode
      // serialization of the origin (Section 6.1 of [RFC6454]) of the push
      // resource URL.
      aud: audience,
      // An "exp" (Expiry) claim MUST be included with the time after which
      // the token expires.
      exp: parseInt(new Date().getTime() / 1000) + 24 * 60 * 60, // seconds, 24hr
      // The "sub" claim SHOULD include a contact URI for the application server as either a
      // "mailto:" (email) [RFC6068] or an "https:" [RFC2818] URI.
      sub: "mailto:webpush@example.com",
    };
    const payload = toBase64Url(new TextEncoder().encode(JSON.stringify(rawPayload)));

    const input = `${header}.${payload}`;
    // https://datatracker.ietf.org/doc/html/rfc7518#section-3.1
    // ES256        | ECDSA using P-256 and SHA-256
    const rawSignature = await crypto.subtle.sign({
      name: "ECDSA",
      namedCurve: "P-256",
      hash: { name: "SHA-256" },
    }, this.#privateKey, new TextEncoder().encode(input));
    const signature = toBase64Url(new Uint8Array(rawSignature));
    return `${input}.${signature}`;
  }

  async generateAuthHeader(audience) {
    // https://datatracker.ietf.org/doc/html/rfc8292#section-3.1
    // The "t" parameter of the "vapid" authentication scheme carries a JWT
    // as described in Section 2.
    const t = await this.#jws(audience);
    // https://datatracker.ietf.org/doc/html/rfc8292#section-3.2
    // The "k" parameter includes an ECDSA public key [FIPS186] in
    // uncompressed form [X9.62] that is encoded using base64url encoding
    // [RFC7515].
    const k = toBase64Url(this.#publicKey)
    return `vapid t=${t},k=${k}`;
  }
};

export async function createVapid() {
  // https://datatracker.ietf.org/doc/html/rfc8292#section-2
  // The signature MUST use ECDSA on the NIST P-256 curve [FIPS186]
  const keys = await crypto.subtle.generateKey({ name: "ECDSA", namedCurve: "P-256" }, true, ["sign"]);
  const publicKey = new Uint8Array(await crypto.subtle.exportKey("raw", keys.publicKey));
  const privateKey = keys.privateKey;
  return new VAPID(publicKey, privateKey);
};
