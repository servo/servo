import base64
import six

def decodebytes(s):
    if six.PY3:
        return base64.decodebytes(six.ensure_binary(s))
    return base64.decodestring(s)

def main(req, res):
    return 404, [(b'Content-Type', b'image/png')], decodebytes(b"iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsQAAA7EAZUrDhsAAAAhSURBVDhPY3wro/KfgQLABKXJBqMGjBoAAqMGDLwBDAwAEsoCTFWunmQAAAAASUVORK5CYII=")
