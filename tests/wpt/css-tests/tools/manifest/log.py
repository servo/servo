import logging

logging.basicConfig()
logger = logging.getLogger("manifest")
logger.setLevel(logging.DEBUG)

def get_logger():
    return logger
