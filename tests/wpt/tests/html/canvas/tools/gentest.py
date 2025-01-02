"""Script invoking the old and the new Canvas test generator."""
# To use this script:
# -make a python virtual environment somewhere (it doesn't matter where)
#   python3 -m venv venv
# -enter the virtual environment
#   source venv/bin/activate
# -install required packages in the venv
#   pip3 install cairocffi jinja2 pyyaml
# -change to the directory with this script and run it
#   python3 gentest.py

from gentestutils import genTestUtils
import gentestutilsunion

genTestUtils('../element', '../element', 'templates.yaml',
             'name2dir-canvas.yaml', False)
genTestUtils('../offscreen', '../offscreen', 'templates.yaml',
             'name2dir-offscreen.yaml', True)
gentestutilsunion.generate_test_files('name2dir.yaml')
