def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=shift-jis")]
    # Shift-JIS bytes for katakana TE SU TO ('test')
    content = bytes([0x83, 0x65, 0x83, 0x58, 0x83, 0x67])

    return headers, content
