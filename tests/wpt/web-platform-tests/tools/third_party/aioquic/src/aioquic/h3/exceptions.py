class H3Error(Exception):
    """
    Base class for HTTP/3 exceptions.
    """


class NoAvailablePushIDError(H3Error):
    """
    There are no available push IDs left.
    """
