import logging
import time
from socket import error as SocketError  # NOQA: N812
from typing import IO

import requests
import urllib3

logger = logging.getLogger(__name__)


def get(url: str) -> requests.Response:
    """Issue GET request to a given URL and return the response."""
    import requests

    logger.debug("GET %s" % url)
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    return resp


def get_download_to_descriptor(fd: IO[bytes], url: str, max_retries: int = 5) -> None:
    """Download an URL in chunks and saves it to a file descriptor (truncating it)
    It doesn't close the descriptor, but flushes it on success.
    It retries the download up to max_retries.
    This function is meant to download big files directly to the disk without
    caching the whole file in memory.
    """
    if max_retries < 1:
        max_retries = 1
    wait = 2
    for current_retry in range(1, max_retries + 1):
        try:
            logger.info("Downloading %s Try %d/%d" % (url, current_retry, max_retries))
            # We may come here in a retry, ensure to truncate fd before start writing.
            fd.seek(0)
            fd.truncate(0)
            resp = get(url)
            for chunk in resp.iter_content(16 * 1024):
                fd.write(chunk)
            fd.flush()
            return
        except (requests.RequestException, SocketError, urllib3.exceptions.HTTPError) as e:
            if current_retry < max_retries:
                # Retry
                logger.error(f"Connection error: {e}. Retrying after {wait}s...")
                time.sleep(wait)
                wait *= 2
            else:
                # Maximum retries or unknown error
                raise
