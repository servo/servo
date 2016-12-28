import sys
sys.path.insert(0, '../../2dcontext/tools/')
import gentestutils
from gentestutils import genTestUtils

genTestUtils('../../offscreen-canvas', '../../offscreen-canvas', 'templates.yaml', 'name2dir.yaml', True)
