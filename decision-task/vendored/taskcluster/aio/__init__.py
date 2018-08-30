""" Python client for Taskcluster """

import logging
import os
from .asyncclient import createSession  # NOQA
from taskcluster.utils import *  # NOQA
from taskcluster.exceptions import *  # NOQA
from ._client_importer import *  # NOQA

log = logging.getLogger(__name__)

if os.environ.get('DEBUG_TASKCLUSTER_CLIENT'):
    log.setLevel(logging.DEBUG)
    if len(log.handlers) == 0:
        log.addHandler(logging.StreamHandler())
log.addHandler(logging.NullHandler())
