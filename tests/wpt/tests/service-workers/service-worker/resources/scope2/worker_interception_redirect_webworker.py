import os

from tools.wpt.utils import load_source

# Use the file from the parent directory.
mod = load_source("_parent", os.path.join(os.path.dirname(os.path.dirname(__file__)),
                                              os.path.basename(__file__)))
main = mod.main
