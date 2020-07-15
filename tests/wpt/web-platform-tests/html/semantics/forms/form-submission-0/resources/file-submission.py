from six import PY3

from wptserve.utils import isomorphic_decode

def main(request, response):
    testinput = request.POST.first(b"testinput")
    if PY3:
        # The test asserts the string representation of this FieldStorage
        # object, but unfortunately the value property has different types in
        # Python 2 and 3. Unify them to native strings.
        testinput.value = isomorphic_decode(testinput.value)
    return ([(b"Content-Type", b"text/html")], u"<script>parent.postMessage(\"" + str(testinput) + u"\", '*');</script>")
