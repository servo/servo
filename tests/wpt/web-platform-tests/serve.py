import sys
import logging

try:
    from tools.serve import serve
except ImportError:
    logging.error("tools.serve not found.  Did you forget to run "
                  '"git submodule update --init --recursive"?')
    sys.exit(2)

def main():
    serve.main()
