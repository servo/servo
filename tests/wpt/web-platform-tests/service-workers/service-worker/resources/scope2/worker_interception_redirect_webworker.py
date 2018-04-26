import os
import imp
# Use the file from the parent directory.
mod = imp.load_source("_parent", os.path.join(os.path.dirname(os.path.dirname(__file__)),
                                              os.path.basename(__file__)))
main = mod.main
