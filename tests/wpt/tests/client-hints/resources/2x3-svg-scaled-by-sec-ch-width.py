def main(request, response):
    """
    Simple handler that responds with an SVG image with width `2 * sec-ch-width`
    and height `3 * sec-ch-width`, or 1x1 if sec-ch-width is not present.
    """

    width = 1
    height = 1

    if b"sec-ch-width" in request.headers:
      sec_ch_width = request.headers.get(b"sec-ch-width").decode()
      width = 2 * int(sec_ch_width)
      height = 3 * int(sec_ch_width)

    response.headers.set(b"Content-Type", b"image/svg+xml")
    response.content = str.encode(f"""<svg
        xmlns="http://www.w3.org/2000/svg"
        xmlns:xlink="http://www.w3.org/1999/xlink"
        width="{width}"
        height="{height}">
      <rect width="100%" height="100%" fill="green" />
    </svg>""")
