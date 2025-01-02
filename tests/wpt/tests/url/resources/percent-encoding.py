import base64
from wptserve.utils import isomorphic_decode

# Use numeric references to let the HTML parser take care of inserting the correct code points
# rather than trying to figure out the necessary bytes for each encoding. (The latter can be
# especially tricky given that Python does not implement the Encoding Standard.)
def numeric_references(input):
    output = b""
    for cp in input:
        output += b"&#x" + format(ord(cp), u"X").encode(u"utf-8") + b";"
    return output

def main(request, response):
    # Undo the "magic" space with + replacement as otherwise base64 decoding will fail.
    value = request.GET.first(b"value").replace(b" ", b"+")
    encoding = request.GET.first(b"encoding")

    output_value = numeric_references(base64.b64decode(value).decode(u"utf-8"))
    return (
        [(b"Content-Type", b"text/html;charset=" + encoding)],
        b"""<!doctype html>
<a href="https://doesnotmatter.invalid/?%s#%s">test</a>
""" % (output_value, output_value))
