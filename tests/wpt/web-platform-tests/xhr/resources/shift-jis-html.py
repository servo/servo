from six import int2byte

def main(request, response):
    headers = [(b"Content-type", b"text/html;charset=shift-jis")]
    # Shift-JIS bytes for katakana TE SU TO ('test')
    content = int2byte(0x83) + int2byte(0x65) + int2byte(0x83) + int2byte(0x58) + int2byte(0x83) + int2byte(0x67)

    return headers, content
