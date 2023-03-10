import pytest


def assert_pdf(data):
    assert data.startswith(b"%PDF-"), "Decoded data starts with the PDF signature"
    assert data.endswith(b"%%EOF\n"), "Decoded data ends with the EOF flag"
