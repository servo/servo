from base64 import decodebytes


def assert_pdf(value):
    data = decodebytes(value.encode())

    assert data.startswith(b"%PDF-"), "Decoded data starts with the PDF signature"
    assert data.endswith(b"%%EOF\n"), "Decoded data ends with the EOF flag"
