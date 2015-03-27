def invert_dict(dict):
    rv = {}
    for key, values in dict.iteritems():
        for value in values:
            if value in rv:
                raise ValueError
            rv[value] = key
    return rv


class HTTPException(Exception):
    def __init__(self, code, message=""):
        self.code = code
        self.message = message
