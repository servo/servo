import json
import base64
import hashlib

# This method decodes the JWT and verifies the signature. If a key is provided,
# that will be used for signature verification. Otherwise, the key sent within
# the JWT payload will be used instead.
# This returns a tuple of (decoded_header, decoded_payload, verify_succeeded).
def decode_jwt(token, key=None):
    try:
        # Decode the header and payload.
        header, payload, signature = token.split('.')
        decoded_header = decode_base64_json(header)
        decoded_payload = decode_base64_json(payload)

        # If decoding failed, return nothing.
        if not decoded_header or not decoded_payload:
            return None, None, False

        # If there is a key passed in (for refresh), use that for checking the signature below.
        # Otherwise (for registration), use the key sent within the JWT to check the signature.
        if key == None:
            key = decoded_header.get('jwk')

        n, e = get_rsa_components(key)

        # Verifying the signature will throw an exception if it fails.
        verify_rs256_signature(header, payload, signature, n, e)
        return decoded_header, decoded_payload, True
    except Exception:
        return None, None, False

def get_rsa_components(jwk_data):
    jwk = json.loads(jwk_data) if isinstance(jwk_data, str) else jwk_data
    key_type = jwk.get("kty")

    if key_type != "RSA":
        raise ValueError(f"Unsupported key type: {key_type}")

    n = int.from_bytes(decode_base64url(jwk["n"]), 'big')
    e = int.from_bytes(decode_base64url(jwk["e"]), 'big')
    return n, e

class SignatureVerificationError(Exception):
    pass

# I2OSP (Integer-to-Octet-String primitive) - RFC 8017 Section 4.1
def i2osp(x, x_len):
    if x >= 256**x_len:
        raise ValueError("integer too large")
    return x.to_bytes(x_len, byteorder='big')

# OS2IP (Octet-String-to-Integer primitive) - RFC 8017 Section 4.2
def os2ip(octet_string):
    return int.from_bytes(octet_string, byteorder='big')

# Verifies an RS256 signature according to RFC 8017 Section 8.2.2.
def verify_rs256_signature(encoded_header, encoded_payload, signature, n, e):
    signature_bytes = decode_base64url(signature)
    M = f"{encoded_header}.{encoded_payload}".encode('ascii')
    k = (n.bit_length() + 7) // 8

    # --- Step 1: Length Checking ---
    if len(signature_bytes) != k:
        raise SignatureVerificationError(
            f"Invalid signature length. Expected {k} bytes, got {len(signature_bytes)}."
        )

    # --- Step 2: RSA Verification (RSAVP1) ---
    # a. Convert signature S to integer representative s
    s = os2ip(signature_bytes)

    # b. Apply RSAVP1: m = s^e mod n
    if s >= n:
        raise SignatureVerificationError("Signature representative out of range (s >= n).")
    m = pow(s, e, n)

    # c. Convert message representative m to encoded message EM
    try:
        EM = i2osp(m, k)
    except ValueError:
        raise SignatureVerificationError("Integer too large for encoded message.")

    # --- Step 3: EMSA-PKCS1-v1_5 Encoding ---
    # We reconstruct what the EM *should* be (EM') and compare it to the EM we recovered.

    # 3.1: Apply Hash (SHA-256)
    sha256 = hashlib.sha256()
    sha256.update(M)
    H = sha256.digest()

    # 3.2: Encode Algorithm ID (DER) for SHA-256
    # (0x)30 31 30 0d 06 09 60 86 48 01 65 03 04 02 01 05 00 04 20 || H
    T = bytes([
        0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01,
        0x65, 0x03, 0x04, 0x02, 0x01, 0x05, 0x00, 0x04, 0x20
    ]) + H
    t_len = len(T)

    # 3.3: Check length requirements
    if k < t_len + 11:
        raise SignatureVerificationError("RSA modulus too short.")

    # 3.4: Generate Padding String PS
    # PS consists of (k - tLen - 3) octets of 0xff
    ps_len = k - t_len - 3
    PS = b'\xff' * ps_len

    # 3.5: Concatenate to form EM'
    # EM' = 0x00 || 0x01 || PS || 0x00 || T
    EM_prime = b'\x00' + b'\x01' + PS + b'\x00' + T

    # --- Step 4: Compare ---
    if EM != EM_prime:
        raise SignatureVerificationError("Invalid signature.")

def add_base64_padding(encoded_data):
    remainder = len(encoded_data) % 4
    if remainder > 0:
        encoded_data += '=' * (4 - remainder)
    return encoded_data

def decode_base64url(encoded_data):
    encoded_data = add_base64_padding(encoded_data)
    encoded_data = encoded_data.replace("-", "+").replace("_", "/")
    return base64.b64decode(encoded_data)

def decode_base64(encoded_data):
    encoded_data = add_base64_padding(encoded_data)
    return base64.urlsafe_b64decode(encoded_data)

def decode_base64_json(encoded_data):
    return json.loads(decode_base64(encoded_data))

def thumbprint_for_jwk(jwk):
    filtered_jwk = None
    if jwk['kty'] == 'RSA':
        filtered_jwk = dict()
        filtered_jwk['kty'] = jwk['kty']
        filtered_jwk['n'] = jwk['n']
        filtered_jwk['e'] = jwk['e']
    elif jwk['kty'] == 'EC':
        filtered_jwk = dict()
        filtered_jwk['kty'] = jwk['kty']
        filtered_jwk['crv'] = jwk['crv']
        filtered_jwk['x'] = jwk['x']
        filtered_jwk['y'] = jwk['y']
    else:
        return None

    serialized_jwk = json.dumps(filtered_jwk, sort_keys=True, separators=(',',':'))

    digest = hashlib.sha256()
    digest.update(serialized_jwk.encode("utf-8"))

    thumbprint_base64 = base64.b64encode(digest.digest(), altchars=b"-_").rstrip(b"=")
    return thumbprint_base64.decode('ascii')