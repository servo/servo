import struct


def _cbor_head(major: int, val: int) -> bytes:
    if val < 24:
        return struct.pack(">B", (major << 5) | val)
    if val <= 0xFF:
        return struct.pack(">BB", (major << 5) | 24, val)
    if val <= 0xFFFF:
        return struct.pack(">BH", (major << 5) | 25, val)
    if val <= 0xFFFFFFFF:
        return struct.pack(">BI", (major << 5) | 26, val)
    return struct.pack(">BQ", (major << 5) | 27, val)


def _cbor_uint(val: int) -> bytes:
    return _cbor_head(0, val)


def _cbor_bytes(data: bytes) -> bytes:
    return _cbor_head(2, len(data)) + data


def _cbor_text(text: str) -> bytes:
    data = text.encode("utf-8")
    return _cbor_head(3, len(data)) + data


def _cbor_array(items: list[bytes]) -> bytes:
    return _cbor_head(4, len(items)) + b"".join(items)


def _cbor_map(entries: list[tuple[bytes, bytes]]) -> bytes:
    sorted_entries = sorted(entries, key=lambda kv: (len(kv[0]), kv[0]))
    return _cbor_head(5, len(sorted_entries)) + b"".join(
        k + v for k, v in sorted_entries
    )


def _build_web_bundle(exchanges: list[tuple[str, dict[str, str], bytes]]) -> bytes:
    encoded_responses = []
    index_locations = []
    offset = 0

    for url, headers, payload in exchanges:
        header_map = _cbor_map(
            [
                (_cbor_bytes(k.encode("utf-8")), _cbor_bytes(v.encode("utf-8")))
                for k, v in headers.items()
            ]
        )
        encoded_response = _cbor_array([_cbor_bytes(header_map), _cbor_bytes(payload)])
        index_locations.append((url, offset, len(encoded_response)))
        offset += len(encoded_response)
        encoded_responses.append(encoded_response)

    initial_offset = len(_cbor_head(4, len(encoded_responses)))
    index_section = _cbor_map(
        [
            (
                _cbor_text(url),
                _cbor_array([_cbor_uint(pos + initial_offset), _cbor_uint(length)]),
            )
            for url, pos, length in index_locations
        ]
    )
    responses_section = _cbor_array(encoded_responses)

    section_lengths = _cbor_array(
        [
            _cbor_text("index"),
            _cbor_uint(len(index_section)),
            _cbor_text("responses"),
            _cbor_uint(len(responses_section)),
        ]
    )
    sections = _cbor_array([index_section, responses_section])

    bundle = bytearray(
        _cbor_array(
            [
                _cbor_bytes("🌐📦".encode("utf-8")),
                _cbor_bytes(b"b2\x00\x00"),
                _cbor_bytes(section_lengths),
                sections,
                _cbor_bytes(b"\x00" * 8),
            ]
        )
    )
    bundle[-8:] = struct.pack(">Q", len(bundle))
    return bytes(bundle)


# Test constants from https://www.rfc-editor.org/rfc/rfc9421.html#name-example-ed25519-test-key
# and https://wicg.github.io/signature-based-sri/#examples.
PUBLIC_KEY = "JrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs="

JSON_BODY = b'{"hello": "world"}'
JSON_DIGEST = "sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:"
JSON_SIGNATURE_INPUT = (
    f'signature=("unencoded-digest";sf);keyid="{PUBLIC_KEY}";tag="sri"'
)
JSON_VALID_SIGNATURE = (
    "signature=:gHim9e5Pk2H7c9BStOmxSmkyc8+ioZgoxynu3d4INAT4dwfj"
    "5LhvaV9DFnEQ9p7C0hzW4o4Qpkm5aApd6WLLCw==:"
)
INVALID_SIGNATURE = (
    "signature=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==:"
)

SCRIPT_EXECUTE_BODY = b"window.hello = `world`;"
SCRIPT_EXECUTE_DIGEST = "sha-256=:PZJ+9CdAAIacg7wfUe4t/RkDQJVKM0mCZ2K7qiRhHFc=:"
SCRIPT_EXECUTE_VALID_SIGNATURE = (
    "signature=:A1wOGCGrcfN34uMe2Umt7hJ6Su1MQFUL1QuT5nmk1R8I761e"
    "XUt2Zv4D5fOt1h1+4DlHPiA1FVwfJLbwlWnpBw==:"
)

SCRIPT_BLOCK_BODY = b"assert_unreached(`This code should not execute.`);"
SCRIPT_BLOCK_DIGEST = "sha-256=:FUSFR1N3vTmSGbI7q9jaMbHq+ogNeBfpznOIufaIfpc=:"


def main(request, response):
    exchanges = [
        (
            "valid-signature.json",
            {
                ":status": "200",
                "content-type": "application/json",
                "unencoded-digest": JSON_DIGEST,
                "signature-input": JSON_SIGNATURE_INPUT,
                "signature": JSON_VALID_SIGNATURE,
            },
            JSON_BODY,
        ),
        (
            "invalid-signature.json",
            {
                ":status": "200",
                "content-type": "application/json",
                "unencoded-digest": JSON_DIGEST,
                "signature-input": JSON_SIGNATURE_INPUT,
                "signature": INVALID_SIGNATURE,
            },
            JSON_BODY,
        ),
        (
            "tampered-body.json",
            {
                ":status": "200",
                "content-type": "application/json",
                "unencoded-digest": JSON_DIGEST,
                "signature-input": JSON_SIGNATURE_INPUT,
                "signature": JSON_VALID_SIGNATURE,
            },
            b'{"hello": "tampered"}',
        ),
        (
            "unsigned.json",
            {
                ":status": "200",
                "content-type": "application/json",
            },
            JSON_BODY,
        ),
        (
            "valid-signature.js",
            {
                ":status": "200",
                "content-type": "application/javascript",
                "unencoded-digest": SCRIPT_EXECUTE_DIGEST,
                "signature-input": JSON_SIGNATURE_INPUT,
                "signature": SCRIPT_EXECUTE_VALID_SIGNATURE,
            },
            SCRIPT_EXECUTE_BODY,
        ),
        (
            "invalid-signature.js",
            {
                ":status": "200",
                "content-type": "application/javascript",
                "unencoded-digest": SCRIPT_BLOCK_DIGEST,
                "signature-input": JSON_SIGNATURE_INPUT,
                "signature": INVALID_SIGNATURE,
            },
            SCRIPT_BLOCK_BODY,
        ),
    ]

    headers = [
        (b"Content-Type", b"application/webbundle"),
        (b"X-Content-Type-Options", b"nosniff"),
    ]
    return (200, headers, _build_web_bundle(exchanges))
