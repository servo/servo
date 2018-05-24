def main(request, response):
    headers = [("Content-type", "text/html;charset=shift-jis")]
    # Shift-JIS bytes for katakana TE SU TO ('test')
    content = chr(0x83) + chr(0x65) + chr(0x83) + chr(0x58) + chr(0x83) + chr(0x67)

    return headers, content
