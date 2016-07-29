# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from win32api import GetModuleHandle
from win32gui import WNDCLASS, RegisterClass, CreateWindow, UpdateWindow
from win32gui import DestroyWindow, LoadIcon, NIF_ICON, NIF_MESSAGE, NIF_TIP
from win32gui import Shell_NotifyIcon, NIM_ADD, NIM_MODIFY, NIF_INFO, NIIF_INFO
import win32con


class WindowsToast:
    def __init__(self):
        # Register window class; it's okay to do this multiple times
        wc = WNDCLASS()
        wc.lpszClassName = 'ServoTaskbarNotification'
        wc.lpfnWndProc = {win32con.WM_DESTROY: self.OnDestroy, }
        self.classAtom = RegisterClass(wc)
        self.hinst = wc.hInstance = GetModuleHandle(None)

    def OnDestroy(self, hwnd, msg, wparam, lparam):
        # We don't have to Shell_NotifyIcon to delete it, since
        # we destroyed
        pass

    def balloon_tip(self, title, msg):
        style = win32con.WS_OVERLAPPED | win32con.WS_SYSMENU
        hwnd = CreateWindow(self.classAtom, "Taskbar", style, 0, 0,
                            win32con.CW_USEDEFAULT, win32con.CW_USEDEFAULT,
                            0, 0, self.hinst, None)
        UpdateWindow(hwnd)

        hicon = LoadIcon(0, win32con.IDI_APPLICATION)

        nid = (hwnd, 0, NIF_ICON | NIF_MESSAGE | NIF_TIP, win32con.WM_USER + 20, hicon, 'Tooltip')
        Shell_NotifyIcon(NIM_ADD, nid)
        nid = (hwnd, 0, NIF_INFO, win32con.WM_USER + 20, hicon, 'Balloon Tooltip', msg, 200, title, NIIF_INFO)
        Shell_NotifyIcon(NIM_MODIFY, nid)

        DestroyWindow(hwnd)
