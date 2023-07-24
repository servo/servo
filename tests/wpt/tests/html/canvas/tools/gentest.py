from gentestutils import genTestUtils
from gentestutilsunion import genTestUtils_union

genTestUtils('../element', '../element', 'templates.yaml',
             'name2dir-canvas.yaml', False)
genTestUtils('../offscreen', '../offscreen', 'templates.yaml',
             'name2dir-offscreen.yaml', True)
genTestUtils_union('name2dir.yaml')
