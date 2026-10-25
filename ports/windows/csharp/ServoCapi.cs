// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Runtime.InteropServices;

namespace Servo.Embedding
{
    public static unsafe class ServoCapi
    {
        private const string LibName = "servo_capi";

        [StructLayout(LayoutKind.Sequential)]
        public struct ServoEventLoopWaker
        {
            public uint StructSize;
            public uint Reserved;
            public delegate* unmanaged[Cdecl]<void> WakeCallback;
        }

        [StructLayout(LayoutKind.Sequential)]
        public struct ServoWebViewDelegate
        {
            public uint StructSize;
            public uint Reserved;
            public void* UserData;
            public delegate* unmanaged[Cdecl]<nint, int, void*, void> NotifyLoadStatusChanged;
            public delegate* unmanaged[Cdecl]<nint, byte*, void*, void> NotifyUrlChanged;
            public delegate* unmanaged[Cdecl]<nint, byte*, void*, void> NotifyPageTitleChanged;
            public delegate* unmanaged[Cdecl]<nint, void*, void> NotifyNewFrameReady;
            public delegate* unmanaged[Cdecl]<nint, void*, void> NotifyClosed;
            public delegate* unmanaged[Cdecl]<nint, byte, void*, void> NotifyFocusChanged;
            public delegate* unmanaged[Cdecl]<nint, void*, void> NotifyGotFocus;
            public delegate* unmanaged[Cdecl]<nint, void*, void> NotifyLoadStarted;
            public delegate* unmanaged[Cdecl]<nint, void*, void> NotifyLoadEnded;
            public delegate* unmanaged[Cdecl]<nint, int, byte*, void*, void> NotifyLoadError;
            public delegate* unmanaged[Cdecl]<nint, int, void*, void> NotifyCursorChanged;
            public delegate* unmanaged[Cdecl]<nint, byte*, void*, void> NotifyStatusTextChanged;
            public delegate* unmanaged[Cdecl]<nint, byte*, void*, void> NotifyFaviconChanged;
            public delegate* unmanaged[Cdecl]<nint, int, byte*, void*, void> ShowConsoleMessage;
        }

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern nint servo_builder_create();

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_builder_set_event_loop_waker(nint builder, ServoEventLoopWaker waker);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern nint servo_builder_build(nint builder);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_free(nint servo);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_spin_event_loop(nint servo);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern nint servo_rendering_context_create_window(void* displayHandle, void* windowHandle, uint width, uint height);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern nint servo_rendering_context_create_software(uint width, uint height);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_rendering_context_free(nint context);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern nint servo_webview_builder_create(nint servo, nint context);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_webview_builder_set_url(nint builder, [MarshalAs(UnmanagedType.LPUTF8Str)] string url);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_webview_builder_set_delegate(nint builder, ServoWebViewDelegate @delegate);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern nint servo_webview_builder_build(nint builder);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_free(nint webview);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_webview_paint(nint webview);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern int servo_webview_load(nint webview, [MarshalAs(UnmanagedType.LPUTF8Str)] string url);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_reload(nint webview);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_stop(nint webview);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern bool servo_webview_can_go_back(nint webview);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern bool servo_webview_can_go_forward(nint webview);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_go_back(nint webview, nuint amount);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_go_forward(nint webview, nuint amount);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_resize(nint webview, uint width, uint height);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        public static extern void servo_webview_notify_theme_change(nint webview, int theme);
    }
}
