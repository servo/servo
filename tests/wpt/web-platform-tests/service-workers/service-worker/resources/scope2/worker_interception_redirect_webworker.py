import os
import sys
# Use the file from the parent directory.
sys.path.append(os.path.dirname(os.path.dirname(__file__)))
from worker_interception_redirect_webworker import main
