config = {
    'mock_target': 'mozilla-centos6-x86_64',
    'mock_packages': ['freetype-devel', 'fontconfig-devel', 'glib2-devel', 'autoconf213', 'git', 'make', 'libX11-devel', 'mesa-libGL-devel', 'freeglut-devel',
                      'xorg-x11-server-devel', 'libXrandr-devel', 'libXi-devel', 'libpng-devel', 'expat-devel', 'gperf', 'gcc473_0moz1', 'libffi-dev', 'libffi6'],
    'mock_files': [('/home/servobld/.ssh', '/home/mock_mozilla/.ssh')],
    'concurrency': 6,
    'add_actions': ['setup-mock'],
    'env': {'PATH': '/tools/gcc-4.7.3-0moz1/bin:%(PATH)s',
            'LIBRARY_PATH': '/tools/gcc-4.7.3-0moz1/lib64',
            'LD_LIBRARY_PATH': '/tools/gcc-4.7.3-0moz1/lib64'},
}
