import logging
import time
from socket import error as SocketError  # NOQA: N812
from typing import IO

import requests
import urllib3

logger = logging.getLogger(__name__)


def get(url: str, timeout: tuple[float, float] = (60, 60)) -> requests.Response:
    """Issue GET request to a given URL and return the response.
    It uses timeout as a (connect, read) tuple in seconds; the read timeout is the
    maximum time between bytes received, not the total time.
    """
    logger.debug("GET %s" % url)
    resp = requests.get(url, stream=True, timeout=timeout)
    resp.raise_for_status()
    return resp


def get_download_to_descriptor(fd: IO[bytes], url: str, max_retries: int = 5,
                               timeout: tuple[float, float] = (60, 60)) -> None:
    """Download a URL in chunks and save it to a file descriptor (truncating it).
    It doesn't close the descriptor, but flushes it on success.
    It retries the download up to max_retries.
    It uses timeout as a (connect, read) tuple in seconds for HTTP request timeouts.
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
            resp = get(url, timeout=timeout)
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
