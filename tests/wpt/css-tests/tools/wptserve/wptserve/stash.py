import uuid

#TODO: Consider expiring values after some fixed time for long-running
#servers


class Stash(object):
    """Key-value store for persisting data across HTTP requests.

    This data store specifically designed for persisting data across
    HTTP requests. It is entirely in-memory so data will not be
    persisted across server restarts.

    This has several unusual properties. Keys are of the form (path,
    uuid), where path is, by default, the path in the HTTP request and
    uuid is a unique id. In addition, the store is write-once, read-once,
    i.e. the value associated with a particular key cannot be changed once
    written and the read operation (called "take") is destructive. Taken together,
    these properties make it difficult for data to accidentally leak
    between different resources or different requests for the same
    resource.

    """

    data = {}

    def __init__(self, default_path):
        self.default_path = default_path

    def put(self, key, value, path=None):
        """Place a value in the stash.

        :param key: A UUID to use as the data's key.
        :param value: The data to store. This can be any python object.
        :param path: The path that has access to read the data (by default
                     the current request path)"""
        if path is None:
            path = self.default_path
        if path not in self.data:
            self.data[path] = PathStash(path)

        self.data[path][key] = value

    def take(self, key, path=None):
        """Remove a value from the stash and return it.

        :param key: A UUID to use as the data's key.
        :param path: The path that has access to read the data (by default
                     the current request path)"""
        if path is None:
            path = self.default_path

        if path in self.data:
            value = self.data[path][key]
        else:
            value = None
        return value


class PathStash(dict):
    def __init__(self, path):
        self.path = path

    def __setitem__(self, key, value):
        key = uuid.UUID(key)
        if value is None:
            raise ValueError("Stash value may not be set to None")
        if key in self:
            raise StashError("Tried to overwrite existing stash value "
                             "for path %s and key %s (old value was %s, new value is %s)" %
                             (self.path, key, self[str(key)], value))
        else:
            dict.__setitem__(self, key, value)

    def __getitem__(self, key):
        key = uuid.UUID(key)
        rv = dict.get(self, key, None)
        if rv is not None:
            del self[key]
        return rv


class StashError(Exception):
    pass
