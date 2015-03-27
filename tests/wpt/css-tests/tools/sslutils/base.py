def get_logger(name="ssl"):
    logger = structured.get_default_logger(name)
    if logger is None:
        logger = structured.structuredlog.StructuredLogger(name)
    return logger

class NoSSLEnvironment(object):
    ssl_enabled = False

    def __init__(self, *args, **kwargs):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *args, **kwargs):
        pass

    def host_cert_path(self, host):
        return None, None

    def ca_cert_path(self):
        return None
