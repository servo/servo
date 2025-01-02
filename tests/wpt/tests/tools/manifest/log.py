import logging

logger = logging.getLogger("manifest")

def enable_debug_logging() -> None:
    logger.setLevel(logging.DEBUG)

def get_logger() -> logging.Logger:
    return logger
