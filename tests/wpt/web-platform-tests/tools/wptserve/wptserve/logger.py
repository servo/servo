class NoOpLogger(object):
    def critical(self, msg):
        pass

    def error(self, msg):
        pass

    def info(self, msg):
        pass

    def warning(self, msg):
        pass

    def debug(self, msg):
        pass

logger = NoOpLogger()
_set_logger = False

def set_logger(new_logger):
    global _set_logger
    if _set_logger:
        raise Exception("Logger must be set at most once")
    global logger
    logger = new_logger
    _set_logger = True

def get_logger():
    return logger
