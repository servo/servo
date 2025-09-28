import json
import base64
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import rsa, padding

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
        public_key = serialization.load_pem_public_key(jwk_to_pem(key))
        # Verifying the signature will throw an exception if it fails.
        verify_rs256_signature(header, payload, signature, public_key)
        return decoded_header, decoded_payload, True
    except Exception:
        return None, None, False

def jwk_to_pem(jwk_data):
    jwk = json.loads(jwk_data) if isinstance(jwk_data, str) else jwk_data
    key_type = jwk.get("kty")

    if key_type != "RSA":
        raise ValueError(f"Unsupported key type: {key_type}")

    n = int.from_bytes(decode_base64url(jwk["n"]), 'big')
    e = int.from_bytes(decode_base64url(jwk["e"]), 'big')
    public_key = rsa.RSAPublicNumbers(e, n).public_key()
    pem_public_key = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    return pem_public_key

def verify_rs256_signature(encoded_header, encoded_payload, signature, public_key):
    message = (f'{encoded_header}.{encoded_payload}').encode('utf-8')
    signature_bytes = decode_base64(signature)
    # This will throw an exception if verification fails.
    public_key.verify(
        signature_bytes,
        message,
        padding.PKCS1v15(),
        hashes.SHA256()
    )

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

    digest = hashes.Hash(hashes.SHA256())
    digest.update(serialized_jwk.encode("utf-8"))

    thumbprint_base64 = base64.b64encode(digest.finalize(), altchars=b"-_").rstrip(b"=")
    return thumbprint_base64.decode('ascii')
