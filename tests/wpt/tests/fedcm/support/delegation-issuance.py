import base64
import json
import re
import time

from cryptography.hazmat.primitives.asymmetric import ed25519

ISSUER_SEED = b"01234567890123456789012345678901"
ERROR_KEY = "f81d4fae-7dec-11d0-a765-00a0c91e6bf6"


def b64url(data):
    if isinstance(data, str):
        data = data.encode("utf-8")
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def extract_client_key(sig_key_header):
    # Parses structured header: sig=hwk;crv="Ed25519";kty="OKP";x="..."
    if not sig_key_header:
        return {"kty": "OKP", "crv": "Ed25519", "x": "dummy_client_key"}

    header_str = sig_key_header.decode("utf-8") if isinstance(
        sig_key_header, bytes) else sig_key_header
    kty_match = re.search(r'kty="?([^";,\s]+)"?', header_str)
    crv_match = re.search(r'crv="?([^";,\s]+)"?', header_str)
    x_match = re.search(r'x="?([^";,\s]+)"?', header_str)

    return {
        "kty": kty_match.group(1) if kty_match else "OKP",
        "crv": crv_match.group(1) if crv_match else "Ed25519",
        "x": x_match.group(1) if x_match else "dummy_client_key"
    }


def main(request, response):
    response.headers.set(b"Content-Type", b"application/json")
    origin = request.headers.get(b"Origin", b"*")
    response.headers.set(b"Access-Control-Allow-Origin", origin)
    response.headers.set(b"Access-Control-Allow-Credentials", b"true")

    if request.method in ("GET", b"GET"):
        if b"set_error" in request.GET:
            err = request.GET.first(b"set_error")
            request.server.stash.take(ERROR_KEY, "/fedcm/support/delegation-issuance.py")
            request.server.stash.put(ERROR_KEY, err, "/fedcm/support/delegation-issuance.py")
            return (200, [], "OK")
        if b"clear_error" in request.GET:
            request.server.stash.take(ERROR_KEY, "/fedcm/support/delegation-issuance.py")
            return (200, [], "OK")
        if b"error" in request.GET and request.GET.first(b"error") == b"500":
            return (500, [], "Internal Server Error")

    error = request.server.stash.take(ERROR_KEY, "/fedcm/support/delegation-issuance.py")
    if error in (b"500", "500") or (b"error" in request.GET and request.GET.first(b"error") == b"500"):
        return (500, [], "Internal Server Error")

    try:
        priv = ed25519.Ed25519PrivateKey.from_private_bytes(ISSUER_SEED)

        sig_key_header = request.headers.get(b"Signature-Key")
        client_jwk = extract_client_key(sig_key_header)

        host = request.url_parts.hostname
        port = request.url_parts.port
        issuer = f"https://{host}:{port}" if port else f"https://{host}"

        email = "john_doe@idp.example"
        try:
            body = json.loads(request.body)
            if "email" in body:
                email = body["email"]
        except Exception:
            pass

        header = {"alg": "EdDSA", "kid": "test_kid", "typ": "evt+jwt"}
        payload = {
            "iss": issuer,
            "email": email,
            "email_verified": True,
            "iat": int(time.time()),
            "cnf": {
                "jwk": client_jwk
            }
        }

        signing_input = f"{b64url(json.dumps(header))}.{b64url(json.dumps(payload))}".encode(
            "ascii")
        sig = priv.sign(signing_input)
        sd_jwt = f"{signing_input.decode('ascii')}.{b64url(sig)}~"

        return json.dumps({"issuance_token": sd_jwt})
    except Exception:
        return (500, [], "Internal Server Error")
