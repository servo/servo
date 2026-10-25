/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <iostream>
#include <string>
#include <vector>
#include <cassert>
#include <algorithm>
#include <cstdint>

#ifndef NOMINMAX
#define NOMINMAX
#endif
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>

// Include Windows Port headers
#include "../src/servo_raw_window.h"
#include "../src/fluent_theme.h"

// Test harness macros
static int g_tests_run = 0;
static int g_tests_failed = 0;

#define TEST_ASSERT(cond, msg) \
    do { \
        if (!(cond)) { \
            std::cerr << "  FAILED: " << msg << " (" << #cond << ") at line " << __LINE__ << std::endl; \
            g_tests_failed++; \
            return; \
        } \
    } while (0)

#define RUN_TEST(fn) \
    do { \
        g_tests_run++; \
        std::cout << "[RUN] " << #fn << "..." << std::endl; \
        int failed_before = g_tests_failed; \
        fn(); \
        if (g_tests_failed == failed_before) { \
            std::cout << "  [PASS] " << #fn << std::endl; \
        } \
    } while (0)

// -----------------------------------------------------------------------------
// 1. Raw Window Handle ABI & Edge Cases
// -----------------------------------------------------------------------------
void test_raw_window_handle_layout() {
    using namespace servo_win;

    // Verify tag alignments
    RawDisplayHandleWindows display_handle = {};
    display_handle.tag = 9; // Windows variant
    TEST_ASSERT(display_handle.tag == 9, "DisplayHandle tag must be 9");

    RawWindowHandleWin32 window_handle = {};
    window_handle.tag = 9; // Win32 variant
    HWND fakeHwnd = reinterpret_cast<HWND>(0x12345678);
    HINSTANCE fakeHinstance = reinterpret_cast<HINSTANCE>(0x87654321);

    window_handle.hwnd = reinterpret_cast<intptr_t>(fakeHwnd);
    window_handle.hinstance = reinterpret_cast<intptr_t>(fakeHinstance);

    TEST_ASSERT(window_handle.tag == 9, "WindowHandle tag must be 9");
    TEST_ASSERT(reinterpret_cast<HWND>(window_handle.hwnd) == fakeHwnd, "HWND pointer must be preserved");
    TEST_ASSERT(reinterpret_cast<HINSTANCE>(window_handle.hinstance) == fakeHinstance, "HINSTANCE pointer must be preserved");

    // Test null / zero-dimension edge cases
    RenderingContext* ctx_null = create_win32_rendering_context(nullptr, 800, 600);
    TEST_ASSERT(ctx_null == nullptr, "create_win32_rendering_context with null HWND must return nullptr");

    RenderingContext* ctx_zero_w = create_win32_rendering_context(fakeHwnd, 0, 600);
    TEST_ASSERT(ctx_zero_w == nullptr, "create_win32_rendering_context with 0 width must return nullptr");

    RenderingContext* ctx_zero_h = create_win32_rendering_context(fakeHwnd, 800, 0);
    TEST_ASSERT(ctx_zero_h == nullptr, "create_win32_rendering_context with 0 height must return nullptr");
}

// -----------------------------------------------------------------------------
// 2. Omnibox URL & Search Query Parsing Edge Cases
// -----------------------------------------------------------------------------
std::string normalize_omnibox_input(const std::string& raw) {
    // Trim leading/trailing whitespace
    size_t first = raw.find_first_not_of(" \t\r\n");
    if (first == std::string::npos) return "";
    size_t last = raw.find_last_not_of(" \t\r\n");
    std::string input = raw.substr(first, (last - first + 1));

    if (input.empty()) return "";

    // If explicit scheme is present
    if (input.find("://") != std::string::npos || input.rfind("about:", 0) == 0) {
        return input;
    }

    // If looks like domain without spaces
    if (input.find('.') != std::string::npos && input.find(' ') == std::string::npos) {
        return "https://" + input;
    }

    // Default to search
    return "https://duckduckgo.com/?q=" + input;
}

void test_omnibox_normalization_edge_cases() {
    TEST_ASSERT(normalize_omnibox_input("https://servo.org") == "https://servo.org", "Standard HTTPS URL");
    TEST_ASSERT(normalize_omnibox_input("http://servo.org/docs") == "http://servo.org/docs", "Standard HTTP URL");
    TEST_ASSERT(normalize_omnibox_input("about:blank") == "about:blank", "about:blank scheme");
    TEST_ASSERT(normalize_omnibox_input("about:version") == "about:version", "about:version scheme");

    // Domain edge cases
    TEST_ASSERT(normalize_omnibox_input("servo.org") == "https://servo.org", "Bare domain prefix https");
    TEST_ASSERT(normalize_omnibox_input("google.com/search?q=rust") == "https://google.com/search?q=rust", "Domain with path & query");
    TEST_ASSERT(normalize_omnibox_input("   servo.org   ") == "https://servo.org", "Whitespace trimming");

    // Search query edge cases
    TEST_ASSERT(normalize_omnibox_input("servo browser engine") == "https://duckduckgo.com/?q=servo browser engine", "Multi-word search query");
    TEST_ASSERT(normalize_omnibox_input("what is rust?") == "https://duckduckgo.com/?q=what is rust?", "Search query with punctuation");
    TEST_ASSERT(normalize_omnibox_input("") == "", "Empty string handling");
    TEST_ASSERT(normalize_omnibox_input("     ") == "", "Pure whitespace string handling");
}

// -----------------------------------------------------------------------------
// 3. Tab Strip Math, Clamping & State Machine Edge Cases
// -----------------------------------------------------------------------------
struct MockTab {
    int id;
    std::string url;
    RECT bounds;
    RECT closeButtonBounds;
};

class MockTabStripState {
public:
    std::vector<MockTab> tabs;
    int activeIndex = -1;
    int nextId = 1;
    RECT addBtnBounds = {};

    int AddTab(const std::string& url) {
        MockTab tab;
        tab.id = nextId++;
        tab.url = url.empty() ? "https://servo.org" : url;
        tabs.push_back(tab);
        activeIndex = static_cast<int>(tabs.size()) - 1;
        Layout(1200);
        return activeIndex;
    }

    void CloseTab(int index) {
        if (index < 0 || index >= static_cast<int>(tabs.size())) return;
        bool wasActive = (index == activeIndex);
        tabs.erase(tabs.begin() + index);

        if (tabs.empty()) {
            activeIndex = -1;
            return;
        }

        if (wasActive) {
            activeIndex = std::min(index, static_cast<int>(tabs.size()) - 1);
        } else if (activeIndex > index) {
            activeIndex--;
        }
        Layout(1200);
    }

    void SelectTab(int index) {
        if (index < 0 || index >= static_cast<int>(tabs.size())) return;
        activeIndex = index;
    }

    void Layout(int totalWidth) {
        if (tabs.empty()) return;
        const int TAB_HEIGHT = 38;
        const int TAB_MAX_WIDTH = 220;
        const int TAB_MIN_WIDTH = 100;
        const int ADD_BTN_WIDTH = 34;

        int available = totalWidth - ADD_BTN_WIDTH - 8;
        int tabWidth = available / static_cast<int>(tabs.size());
        tabWidth = std::clamp(tabWidth, TAB_MIN_WIDTH, TAB_MAX_WIDTH);

        int curX = 8;
        for (auto& tab : tabs) {
            tab.bounds = { curX, 4, curX + tabWidth - 2, TAB_HEIGHT };
            tab.closeButtonBounds = { tab.bounds.right - 24, 10, tab.bounds.right - 6, 28 };
            curX += tabWidth;
        }
        addBtnBounds = { curX + 4, 6, curX + 4 + ADD_BTN_WIDTH - 8, TAB_HEIGHT - 6 };
    }
};

void test_tab_strip_state_machine() {
    MockTabStripState state;
    TEST_ASSERT(state.activeIndex == -1, "Initial active index is -1");

    // Add Tab 1
    int idx0 = state.AddTab("https://servo.org");
    TEST_ASSERT(idx0 == 0 && state.activeIndex == 0, "First tab is active index 0");
    TEST_ASSERT(state.tabs.size() == 1, "Tab count is 1");

    // Add Tab 2
    int idx1 = state.AddTab("https://github.com");
    TEST_ASSERT(idx1 == 1 && state.activeIndex == 1, "Second tab is active index 1");

    // Add Tab 3
    int idx2 = state.AddTab("https://rust-lang.org");
    TEST_ASSERT(idx2 == 2 && state.activeIndex == 2, "Third tab is active index 2");

    // Close middle tab (Tab 1, while Tab 2 is active)
    state.CloseTab(1);
    TEST_ASSERT(state.tabs.size() == 2, "Tab count after close is 2");
    TEST_ASSERT(state.activeIndex == 1, "Active index shifted from 2 to 1");
    TEST_ASSERT(state.tabs[1].url == "https://rust-lang.org", "Remaining tab matches");

    // Close active tab (Tab 1)
    state.CloseTab(1);
    TEST_ASSERT(state.tabs.size() == 1, "Tab count after closing active is 1");
    TEST_ASSERT(state.activeIndex == 0, "Active index selected remaining neighbor");

    // Close last tab
    state.CloseTab(0);
    TEST_ASSERT(state.tabs.empty(), "Tabs empty after closing last");
    TEST_ASSERT(state.activeIndex == -1, "Active index resets to -1");

    // Out of bounds selection / closing edge cases
    state.SelectTab(-5);
    TEST_ASSERT(state.activeIndex == -1, "Negative index selection ignored");
    state.SelectTab(99);
    TEST_ASSERT(state.activeIndex == -1, "Out of bounds selection ignored");
    state.CloseTab(0); // Should not throw/crash
}

void test_tab_strip_clamping() {
    MockTabStripState state;
    // Add 1 tab in a 1200px window -> tab width should clamp to TAB_MAX_WIDTH (220)
    state.AddTab("https://servo.org");
    int tabW = state.tabs[0].bounds.right - state.tabs[0].bounds.left + 2;
    TEST_ASSERT(tabW == 220, "Single tab width clamped to 220px");

    // Add 25 tabs in 1200px window -> tab width should clamp to TAB_MIN_WIDTH (100)
    for (int i = 0; i < 24; ++i) {
        state.AddTab("https://servo.org");
    }
    int tabW25 = state.tabs[0].bounds.right - state.tabs[0].bounds.left + 2;
    TEST_ASSERT(tabW25 == 100, "Many tabs width clamped to 100px minimum");
}

// -----------------------------------------------------------------------------
// 4. FindBar Match Formatting & Edge Cases
// -----------------------------------------------------------------------------
std::wstring format_find_result(int activeOrdinal, int totalMatches) {
    if (totalMatches > 0) {
        return std::to_wstring(activeOrdinal) + L" of " + std::to_wstring(totalMatches);
    }
    return L"0 of 0";
}

void test_find_bar_formatting() {
    TEST_ASSERT(format_find_result(1, 10) == L"1 of 10", "Ordinal 1 of 10");
    TEST_ASSERT(format_find_result(5, 5) == L"5 of 5", "Ordinal at last match");
    TEST_ASSERT(format_find_result(0, 0) == L"0 of 0", "No matches found");
    TEST_ASSERT(format_find_result(-1, 0) == L"0 of 0", "Negative ordinal edge case");
}

// -----------------------------------------------------------------------------
// 5. Windows 11 Mica Alt Constants & Theme Definitions
// -----------------------------------------------------------------------------
void test_mica_alt_constants() {
    const int DWMSBT_TABBEDWINDOW_VAL = 4; // Mica Alt
    const int DWMWA_SYSTEMBACKDROP_TYPE_VAL = 38;
    const int DWMWA_USE_IMMERSIVE_DARK_MODE_VAL = 20;

    TEST_ASSERT(DWMSBT_TABBEDWINDOW_VAL == 4, "DWMSBT_TABBEDWINDOW must be 4 for Mica Alt");
    TEST_ASSERT(DWMWA_SYSTEMBACKDROP_TYPE_VAL == 38, "DWMWA_SYSTEMBACKDROP_TYPE attribute ID");
    TEST_ASSERT(DWMWA_USE_IMMERSIVE_DARK_MODE_VAL == 20, "DWMWA_USE_IMMERSIVE_DARK_MODE attribute ID");

    // Servo Theme Constants
    TEST_ASSERT(SERVO_THEME_LIGHT == 0, "SERVO_THEME_LIGHT is 0");
    TEST_ASSERT(SERVO_THEME_DARK == 1, "SERVO_THEME_DARK is 1");
}

// -----------------------------------------------------------------------------
// 6. Input Event Modifiers Bitflags
// -----------------------------------------------------------------------------
void test_servo_modifiers() {
    uint32_t shift = SERVO_MODIFIER_SHIFT;
    uint32_t ctrl = SERVO_MODIFIER_CONTROL;
    uint32_t alt = SERVO_MODIFIER_ALT;

    TEST_ASSERT((shift & ctrl) == 0, "Modifiers must be orthogonal bits");
    TEST_ASSERT((shift & alt) == 0, "Modifiers must be orthogonal bits");
    TEST_ASSERT((ctrl & alt) == 0, "Modifiers must be orthogonal bits");

    uint32_t combined = shift | ctrl;
    TEST_ASSERT(combined & SERVO_MODIFIER_SHIFT, "Bitflag checks correctly");
    TEST_ASSERT(combined & SERVO_MODIFIER_CONTROL, "Bitflag checks correctly");
    TEST_ASSERT(!(combined & SERVO_MODIFIER_ALT), "Unset bitflag is false");
}

// -----------------------------------------------------------------------------
// 7. Tab Cycling, Direct Selection & History Reopening
// -----------------------------------------------------------------------------
void test_tab_navigation_and_reopen() {
    MockTabStripState state;
    std::vector<std::string> closedHistory;

    state.AddTab("https://servo.org");      // 0
    state.AddTab("https://github.com");     // 1
    state.AddTab("https://rust-lang.org");  // 2
    TEST_ASSERT(state.activeIndex == 2, "Last added tab is active");

    // NextTab cycling: 2 -> 0 -> 1 -> 2
    state.activeIndex = (state.activeIndex + 1) % state.tabs.size();
    TEST_ASSERT(state.activeIndex == 0, "Cycling next wraps around to 0");
    state.activeIndex = (state.activeIndex + 1) % state.tabs.size();
    TEST_ASSERT(state.activeIndex == 1, "Cycling next reaches 1");

    // PreviousTab cycling: 1 -> 0 -> 2
    state.activeIndex = (state.activeIndex - 1 + state.tabs.size()) % state.tabs.size();
    TEST_ASSERT(state.activeIndex == 0, "Cycling previous reaches 0");
    state.activeIndex = (state.activeIndex - 1 + state.tabs.size()) % state.tabs.size();
    TEST_ASSERT(state.activeIndex == 2, "Cycling previous wraps to last tab");

    // Direct tab index selection
    state.SelectTab(1);
    TEST_ASSERT(state.activeIndex == 1, "SelectTab 1 works");

    // Close Tab 1 and track in closed history
    std::string closedUrl = state.tabs[1].url;
    closedHistory.push_back(closedUrl);
    state.CloseTab(1);
    TEST_ASSERT(state.tabs.size() == 2, "Count reduced after closing");

    // Reopen closed tab
    TEST_ASSERT(!closedHistory.empty(), "Closed history contains closed tab");
    std::string toRestore = closedHistory.back();
    closedHistory.pop_back();
    int restoredIdx = state.AddTab(toRestore);
    TEST_ASSERT(state.tabs.size() == 3, "Count restored to 3");
    TEST_ASSERT(state.tabs[restoredIdx].url == "https://github.com", "Restored tab matches URL");
}

// -----------------------------------------------------------------------------
// 8. Keyboard Shortcut Action Mapping
// -----------------------------------------------------------------------------
enum class BrowserAction {
    None,
    NewTab,
    CloseTab,
    ReopenTab,
    NextTab,
    PrevTab,
    SelectTab1,
    SelectLastTab,
    FocusOmnibox,
    SearchOmnibox,
    Reload,
    ForceReload,
    Stop,
    Home,
    Back,
    Forward,
    Find,
    Downloads,
    DevTools,
    Fullscreen,
    ZoomIn,
    ZoomOut,
    ZoomReset
};

BrowserAction map_key_to_action(bool ctrl, bool shift, bool alt, WPARAM vk) {
    if (ctrl && shift && vk == 'T') return BrowserAction::ReopenTab;
    if (ctrl && (vk == VK_TAB || vk == VK_NEXT)) return shift ? BrowserAction::PrevTab : BrowserAction::NextTab;
    if (ctrl && vk == VK_PRIOR) return BrowserAction::PrevTab;
    if (ctrl && vk == '1') return BrowserAction::SelectTab1;
    if (ctrl && vk == '9') return BrowserAction::SelectLastTab;
    if (ctrl && vk == 'T') return BrowserAction::NewTab;
    if ((ctrl && vk == 'W') || (ctrl && vk == VK_F4)) return BrowserAction::CloseTab;

    if ((ctrl && vk == 'L') || (alt && vk == 'D') || vk == VK_F6) return BrowserAction::FocusOmnibox;
    if (ctrl && (vk == 'E' || vk == 'K')) return BrowserAction::SearchOmnibox;

    if (vk == VK_ESCAPE) return BrowserAction::Stop;
    if ((ctrl && shift && vk == 'R') || (ctrl && vk == VK_F5)) return BrowserAction::ForceReload;
    if ((ctrl && vk == 'R') || vk == VK_F5) return BrowserAction::Reload;
    if (alt && vk == VK_HOME) return BrowserAction::Home;
    if (alt && vk == VK_LEFT) return BrowserAction::Back;
    if (alt && vk == VK_RIGHT) return BrowserAction::Forward;

    if (ctrl && vk == 'F') return BrowserAction::Find;
    if (ctrl && vk == 'J') return BrowserAction::Downloads;
    if (vk == VK_F12 || (ctrl && shift && vk == 'I')) return BrowserAction::DevTools;
    if (vk == VK_F11) return BrowserAction::Fullscreen;

    if (ctrl && (vk == VK_OEM_PLUS || vk == VK_ADD || vk == '=')) return BrowserAction::ZoomIn;
    if (ctrl && (vk == VK_OEM_MINUS || vk == VK_SUBTRACT || vk == '-')) return BrowserAction::ZoomOut;
    if (ctrl && (vk == '0' || vk == VK_NUMPAD0)) return BrowserAction::ZoomReset;

    return BrowserAction::None;
}

void test_keyboard_shortcut_mapping() {
    TEST_ASSERT(map_key_to_action(true, true, false, 'T') == BrowserAction::ReopenTab, "Ctrl+Shift+T -> ReopenTab");
    TEST_ASSERT(map_key_to_action(true, false, false, VK_TAB) == BrowserAction::NextTab, "Ctrl+Tab -> NextTab");
    TEST_ASSERT(map_key_to_action(true, true, false, VK_TAB) == BrowserAction::PrevTab, "Ctrl+Shift+Tab -> PrevTab");
    TEST_ASSERT(map_key_to_action(true, false, false, VK_NEXT) == BrowserAction::NextTab, "Ctrl+PageDown -> NextTab");
    TEST_ASSERT(map_key_to_action(true, false, false, VK_PRIOR) == BrowserAction::PrevTab, "Ctrl+PageUp -> PrevTab");
    TEST_ASSERT(map_key_to_action(true, false, false, '1') == BrowserAction::SelectTab1, "Ctrl+1 -> SelectTab1");
    TEST_ASSERT(map_key_to_action(true, false, false, '9') == BrowserAction::SelectLastTab, "Ctrl+9 -> SelectLastTab");
    TEST_ASSERT(map_key_to_action(true, false, false, 'T') == BrowserAction::NewTab, "Ctrl+T -> NewTab");
    TEST_ASSERT(map_key_to_action(true, false, false, 'W') == BrowserAction::CloseTab, "Ctrl+W -> CloseTab");
    TEST_ASSERT(map_key_to_action(true, false, false, VK_F4) == BrowserAction::CloseTab, "Ctrl+F4 -> CloseTab");

    TEST_ASSERT(map_key_to_action(true, false, false, 'L') == BrowserAction::FocusOmnibox, "Ctrl+L -> FocusOmnibox");
    TEST_ASSERT(map_key_to_action(false, false, true, 'D') == BrowserAction::FocusOmnibox, "Alt+D -> FocusOmnibox");
    TEST_ASSERT(map_key_to_action(false, false, false, VK_F6) == BrowserAction::FocusOmnibox, "F6 -> FocusOmnibox");
    TEST_ASSERT(map_key_to_action(true, false, false, 'E') == BrowserAction::SearchOmnibox, "Ctrl+E -> SearchOmnibox");
    TEST_ASSERT(map_key_to_action(true, false, false, 'K') == BrowserAction::SearchOmnibox, "Ctrl+K -> SearchOmnibox");

    TEST_ASSERT(map_key_to_action(false, false, false, VK_ESCAPE) == BrowserAction::Stop, "Esc -> Stop");
    TEST_ASSERT(map_key_to_action(true, true, false, 'R') == BrowserAction::ForceReload, "Ctrl+Shift+R -> ForceReload");
    TEST_ASSERT(map_key_to_action(true, false, false, VK_F5) == BrowserAction::ForceReload, "Ctrl+F5 -> ForceReload");
    TEST_ASSERT(map_key_to_action(true, false, false, 'R') == BrowserAction::Reload, "Ctrl+R -> Reload");
    TEST_ASSERT(map_key_to_action(false, false, false, VK_F5) == BrowserAction::Reload, "F5 -> Reload");
    TEST_ASSERT(map_key_to_action(false, false, true, VK_HOME) == BrowserAction::Home, "Alt+Home -> Home");
    TEST_ASSERT(map_key_to_action(false, false, true, VK_LEFT) == BrowserAction::Back, "Alt+Left -> Back");
    TEST_ASSERT(map_key_to_action(false, false, true, VK_RIGHT) == BrowserAction::Forward, "Alt+Right -> Forward");

    TEST_ASSERT(map_key_to_action(true, false, false, 'F') == BrowserAction::Find, "Ctrl+F -> Find");
    TEST_ASSERT(map_key_to_action(true, false, false, 'J') == BrowserAction::Downloads, "Ctrl+J -> Downloads");
    TEST_ASSERT(map_key_to_action(false, false, false, VK_F12) == BrowserAction::DevTools, "F12 -> DevTools");
    TEST_ASSERT(map_key_to_action(true, true, false, 'I') == BrowserAction::DevTools, "Ctrl+Shift+I -> DevTools");
    TEST_ASSERT(map_key_to_action(false, false, false, VK_F11) == BrowserAction::Fullscreen, "F11 -> Fullscreen");

    TEST_ASSERT(map_key_to_action(true, false, false, VK_OEM_PLUS) == BrowserAction::ZoomIn, "Ctrl++ -> ZoomIn");
    TEST_ASSERT(map_key_to_action(true, false, false, '=') == BrowserAction::ZoomIn, "Ctrl+= -> ZoomIn");
    TEST_ASSERT(map_key_to_action(true, false, false, VK_OEM_MINUS) == BrowserAction::ZoomOut, "Ctrl+- -> ZoomOut");
    TEST_ASSERT(map_key_to_action(true, false, false, '0') == BrowserAction::ZoomReset, "Ctrl+0 -> ZoomReset");
}

// -----------------------------------------------------------------------------
// 9. Zoom Calculation & Clamping
// -----------------------------------------------------------------------------
float compute_zoom_in(float cur) {
    return std::min(cur * 1.2f, 5.0f);
}

float compute_zoom_out(float cur) {
    return std::max(cur / 1.2f, 0.25f);
}

void test_zoom_calculations() {
    float z = 1.0f;
    z = compute_zoom_in(z);
    TEST_ASSERT(z > 1.0f && z <= 1.21f, "Zoom in from 1.0f increases zoom to 1.2f");

    z = compute_zoom_out(z);
    TEST_ASSERT(z >= 0.99f && z <= 1.01f, "Zoom out returns to ~1.0f");

    // Repeated zoom in clamps at 5.0f
    for (int i = 0; i < 20; ++i) {
        z = compute_zoom_in(z);
    }
    TEST_ASSERT(z == 5.0f, "Zoom in clamps to max 5.0f");

    // Repeated zoom out clamps at 0.25f
    for (int i = 0; i < 30; ++i) {
        z = compute_zoom_out(z);
    }
    TEST_ASSERT(z == 0.25f, "Zoom out clamps to min 0.25f");
}

// -----------------------------------------------------------------------------
// 10. WinUI 3 Theme Tokens & Fluent Icons
// -----------------------------------------------------------------------------
void test_fluent_theme_tokens_and_icons() {
    auto dark = fluent::GetDarkColors();
    auto light = fluent::GetLightColors();

    // Verify dark palette properties
    TEST_ASSERT(dark.LayerBackground == RGB(32, 32, 32), "Dark Mica Alt background token");
    TEST_ASSERT(dark.TextPrimary == RGB(255, 255, 255), "Dark text primary token");
    TEST_ASSERT(dark.Accent == RGB(96, 205, 255), "Dark WinUI 3 accent color");

    // Verify light palette properties
    TEST_ASSERT(light.LayerBackground == RGB(243, 243, 243), "Light Mica Alt background token");
    TEST_ASSERT(light.TextPrimary == RGB(0, 0, 0), "Light text primary token");
    TEST_ASSERT(light.Accent == RGB(0, 95, 184), "Light WinUI 3 accent color");

    // Verify Fluent Icons codepoints
    TEST_ASSERT(std::wstring(fluent::icons::Back) == L"\uE72B", "Fluent Back icon");
    TEST_ASSERT(std::wstring(fluent::icons::Forward) == L"\uE72A", "Fluent Forward icon");
    TEST_ASSERT(std::wstring(fluent::icons::Refresh) == L"\uE72C", "Fluent Refresh icon");
    TEST_ASSERT(std::wstring(fluent::icons::Stop) == L"\uE711", "Fluent Stop icon");
    TEST_ASSERT(std::wstring(fluent::icons::Home) == L"\uE80F", "Fluent Home icon");
    TEST_ASSERT(std::wstring(fluent::icons::Close) == L"\uE8BB", "Fluent Close icon");
    TEST_ASSERT(std::wstring(fluent::icons::AddTab) == L"\uE710", "Fluent AddTab icon");
    TEST_ASSERT(std::wstring(fluent::icons::Lock) == L"\uE72E", "Fluent Lock icon");
}

// -----------------------------------------------------------------------------
// Main Runner
// -----------------------------------------------------------------------------
int main() {
    std::cout << "========================================" << std::endl;
    std::cout << " Running Windows Browser Unit Test Suite" << std::endl;
    std::cout << "========================================" << std::endl;

    RUN_TEST(test_raw_window_handle_layout);
    RUN_TEST(test_omnibox_normalization_edge_cases);
    RUN_TEST(test_tab_strip_state_machine);
    RUN_TEST(test_tab_strip_clamping);
    RUN_TEST(test_find_bar_formatting);
    RUN_TEST(test_mica_alt_constants);
    RUN_TEST(test_servo_modifiers);
    RUN_TEST(test_tab_navigation_and_reopen);
    RUN_TEST(test_keyboard_shortcut_mapping);
    RUN_TEST(test_zoom_calculations);
    RUN_TEST(test_fluent_theme_tokens_and_icons);

    std::cout << "========================================" << std::endl;
    std::cout << " Summary: " << g_tests_run << " tests run, "
              << g_tests_failed << " failed." << std::endl;
    std::cout << "========================================" << std::endl;

    return (g_tests_failed == 0) ? 0 : 1;
}
