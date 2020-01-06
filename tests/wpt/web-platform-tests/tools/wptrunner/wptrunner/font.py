import ctypes
import os
import platform
import plistlib

from shutil import copy2, rmtree
from subprocess import call, check_output

HERE = os.path.split(__file__)[0]
SYSTEM = platform.system().lower()


class FontInstaller(object):
    def __init__(self, logger, font_dir=None, **fonts):
        self.logger = logger
        self.font_dir = font_dir
        self.installed_fonts = False
        self.created_dir = False
        self.fonts = fonts

    def __call__(self, env_options=None, env_config=None):
        return self

    def __enter__(self):
        for _, font_path in self.fonts.items():
            font_name = font_path.split('/')[-1]
            install = getattr(self, 'install_%s_font' % SYSTEM, None)
            if not install:
                self.logger.warning('Font installation not supported on %s' % SYSTEM)
                return False
            if install(font_name, font_path):
                self.installed_fonts = True
                self.logger.info('Installed font: %s' % font_name)
            else:
                self.logger.warning('Unable to install font: %s' % font_name)

    def __exit__(self, exc_type, exc_val, exc_tb):
        if not self.installed_fonts:
            return False

        for _, font_path in self.fonts.items():
            font_name = font_path.split('/')[-1]
            remove = getattr(self, 'remove_%s_font' % SYSTEM, None)
            if not remove:
                self.logger.warning('Font removal not supported on %s' % SYSTEM)
                return False
            if remove(font_name, font_path):
                self.logger.info('Removed font: %s' % font_name)
            else:
                self.logger.warning('Unable to remove font: %s' % font_name)

    def install_linux_font(self, font_name, font_path):
        if not self.font_dir:
            self.font_dir = os.path.join(os.path.expanduser('~'), '.fonts')
        if not os.path.exists(self.font_dir):
            os.makedirs(self.font_dir)
            self.created_dir = True
        if not os.path.exists(os.path.join(self.font_dir, font_name)):
            copy2(font_path, self.font_dir)
        try:
            fc_cache_returncode = call('fc-cache')
            return not fc_cache_returncode
        except OSError:  # If fontconfig doesn't exist, return False
            self.logger.error('fontconfig not available on this Linux system.')
            return False

    def install_darwin_font(self, font_name, font_path):
        if not self.font_dir:
            self.font_dir = os.path.join(os.path.expanduser('~'),
                                         'Library/Fonts')
        if not os.path.exists(self.font_dir):
            os.makedirs(self.font_dir)
            self.created_dir = True
        installed_font_path = os.path.join(self.font_dir, font_name)
        if not os.path.exists(installed_font_path):
            copy2(font_path, self.font_dir)

        # Per https://github.com/web-platform-tests/results-collection/issues/218
        # installing Ahem on macOS is flaky, so check if it actually installed
        fonts = check_output(['/usr/sbin/system_profiler', '-xml', 'SPFontsDataType'])
        fonts = plistlib.readPlistFromString(fonts)
        assert len(fonts) == 1
        for font in fonts[0]['_items']:
            if font['path'] == installed_font_path:
                return True
        return False

    def install_windows_font(self, _, font_path):
        hwnd_broadcast = 0xFFFF
        wm_fontchange = 0x001D

        gdi32 = ctypes.WinDLL('gdi32')
        if gdi32.AddFontResourceW(font_path):
            from ctypes import wintypes
            wparam = 0
            lparam = 0
            SendNotifyMessageW = ctypes.windll.user32.SendNotifyMessageW
            SendNotifyMessageW.argtypes = [wintypes.HANDLE, wintypes.UINT,
                                           wintypes.WPARAM, wintypes.LPARAM]
            return bool(SendNotifyMessageW(hwnd_broadcast, wm_fontchange,
                                           wparam, lparam))

    def remove_linux_font(self, font_name, _):
        if self.created_dir:
            rmtree(self.font_dir)
        else:
            os.remove('%s/%s' % (self.font_dir, font_name))
        try:
            fc_cache_returncode = call('fc-cache')
            return not fc_cache_returncode
        except OSError:  # If fontconfig doesn't exist, return False
            self.logger.error('fontconfig not available on this Linux system.')
            return False

    def remove_darwin_font(self, font_name, _):
        if self.created_dir:
            rmtree(self.font_dir)
        else:
            os.remove(os.path.join(self.font_dir, font_name))
        return True

    def remove_windows_font(self, _, font_path):
        hwnd_broadcast = 0xFFFF
        wm_fontchange = 0x001D

        gdi32 = ctypes.WinDLL('gdi32')
        if gdi32.RemoveFontResourceW(font_path):
            from ctypes import wintypes
            wparam = 0
            lparam = 0
            SendNotifyMessageW = ctypes.windll.user32.SendNotifyMessageW
            SendNotifyMessageW.argtypes = [wintypes.HANDLE, wintypes.UINT,
                                           wintypes.WPARAM, wintypes.LPARAM]
            return bool(SendNotifyMessageW(hwnd_broadcast, wm_fontchange,
                                           wparam, lparam))
