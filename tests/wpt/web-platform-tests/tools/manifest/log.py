import logging

logger = logging.getLogger("manifest")

def enable_debug_logging():
    # type: () -> None
    logger.setLevel(logging.DEBUG)

def get_logger():
    # type: () -> logging.Logger
    return logger
