from gentestutils import genTestUtils

genTestUtils('../element', '../element', 'templates.yaml', 'name2dir.yaml', False)
genTestUtils('../offscreen', '../offscreen', 'templates-offscreen.yaml', 'name2dir-offscreen.yaml', True)
