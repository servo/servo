import logging
import sys

logger = logging.getLogger("manifest")
logger.setLevel(logging.DEBUG)
handler = logging.StreamHandler(sys.stdout)
formatter = logging.Formatter(logging.BASIC_FORMAT, None)
handler.setFormatter(formatter)
logger.addHandler(handler)

def get_logger():
    return logger
