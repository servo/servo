/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#pragma once

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>

namespace fluent {

// Segoe Fluent Icons / Segoe MDL2 Assets Unicode Codepoints
namespace icons {
    constexpr const wchar_t* Back = L"\uE72B";
    constexpr const wchar_t* Forward = L"\uE72A";
    constexpr const wchar_t* Refresh = L"\uE72C";
    constexpr const wchar_t* Stop = L"\uE711";
    constexpr const wchar_t* Home = L"\uE80F";
    constexpr const wchar_t* AddTab = L"\uE710";
    constexpr const wchar_t* Close = L"\uE8BB";
    constexpr const wchar_t* Search = L"\uE721";
    constexpr const wchar_t* Lock = L"\uE72E";
    constexpr const wchar_t* Downloads = L"\uE896";
    constexpr const wchar_t* Find = L"\uE773";
    constexpr const wchar_t* More = L"\uE712";
    constexpr const wchar_t* ChevronUp = L"\uE70E";
    constexpr const wchar_t* ChevronDown = L"\uE70D";
}

// WinUI 3 Fluent Design System Color Tokens
struct ThemeColors {
    COLORREF LayerBackground;
    COLORREF TabBackgroundActive;
    COLORREF TabBackgroundInactive;
    COLORREF TabBackgroundHover;
    COLORREF TabBorder;
    COLORREF ControlBackground;
    COLORREF ControlBackgroundHover;
    COLORREF ControlBackgroundPressed;
    COLORREF ControlBorder;
    COLORREF ControlBorderFocused;
    COLORREF TextPrimary;
    COLORREF TextSecondary;
    COLORREF TextTertiary;
    COLORREF Accent;
    COLORREF CardBackground;
    COLORREF CardBorder;
};

inline ThemeColors GetDarkColors() {
    ThemeColors c = {};
    c.LayerBackground = RGB(32, 32, 32);            // Mica Alt dark tint
    c.TabBackgroundActive = RGB(45, 45, 45);        // Active tab card
    c.TabBackgroundInactive = RGB(32, 32, 32);      // Blends with Mica
    c.TabBackgroundHover = RGB(40, 40, 40);         // Hovered tab
    c.TabBorder = RGB(60, 60, 60);
    c.ControlBackground = RGB(45, 45, 45);
    c.ControlBackgroundHover = RGB(55, 55, 55);
    c.ControlBackgroundPressed = RGB(40, 40, 40);
    c.ControlBorder = RGB(65, 65, 65);
    c.ControlBorderFocused = RGB(96, 205, 255);     // WinUI 3 Accent
    c.TextPrimary = RGB(255, 255, 255);
    c.TextSecondary = RGB(197, 197, 197);
    c.TextTertiary = RGB(140, 140, 140);
    c.Accent = RGB(96, 205, 255);                   // SystemAccentColorDark
    c.CardBackground = RGB(43, 43, 43);
    c.CardBorder = RGB(60, 60, 60);
    return c;
}

inline ThemeColors GetLightColors() {
    ThemeColors c = {};
    c.LayerBackground = RGB(243, 243, 243);         // Mica Alt light tint
    c.TabBackgroundActive = RGB(255, 255, 255);     // Active tab white card
    c.TabBackgroundInactive = RGB(243, 243, 243);   // Blends with Mica
    c.TabBackgroundHover = RGB(236, 236, 236);
    c.TabBorder = RGB(225, 225, 225);
    c.ControlBackground = RGB(255, 255, 255);
    c.ControlBackgroundHover = RGB(247, 247, 247);
    c.ControlBackgroundPressed = RGB(240, 240, 240);
    c.ControlBorder = RGB(215, 215, 215);
    c.ControlBorderFocused = RGB(0, 95, 184);       // WinUI 3 Accent
    c.TextPrimary = RGB(0, 0, 0);
    c.TextSecondary = RGB(93, 93, 93);
    c.TextTertiary = RGB(150, 150, 150);
    c.Accent = RGB(0, 95, 184);                     // SystemAccentColorLight
    c.CardBackground = RGB(255, 255, 255);
    c.CardBorder = RGB(229, 229, 229);
    return c;
}

inline HFONT CreateFluentFont(int height, int weight = FW_NORMAL, bool italic = false) {
    return CreateFontW(
        height, 0, 0, 0, weight, italic ? TRUE : FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE,
        L"Segoe UI Variable Text"
    );
}

inline HFONT CreateFluentIconFont(int height) {
    // Falls back gracefully: Segoe Fluent Icons (Win 11) -> Segoe MDL2 Assets (Win 10) -> Segoe UI Symbol
    return CreateFontW(
        height, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE,
        L"Segoe Fluent Icons"
    );
}

} // namespace fluent
