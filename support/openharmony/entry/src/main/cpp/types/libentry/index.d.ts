export interface InitOpts {
    url: string;
    resourceDir: string,
    commandlineArgs: string,
}

export const loadURL: (url: string) => void;
export const goBack: () => void;
export const goForward: () => void;
export const registerURLcallback: (callback: (url: string) => void) => void;
export const registerTerminateCallback: (callback: () => void) => void;
export const registerPromptToastCallback: (callback: (msg: string) => void) => void;
/** Title of the currently focused webview. */
export const registerTitleCallback: (callback: (title: string) => void) => void;
/** bit 0 = can go back, bit 1 = can go forward. */
export const registerNavStateCallback: (callback: (state: number) => void) => void;
/** 1 = started, 2 = head parsed, 3 = complete. */
export const registerLoadStatusCallback: (callback: (status: number) => void) => void;
/** Small integer id mapped from `servo::Cursor` (see `host_callbacks::cursor_id`). */
export const registerCursorCallback: (callback: (id: number) => void) => void;
/** 1 = new tab, 2 = close tab, 3 = focus address bar, 4 = reload, 5 = next tab. */
export const registerShortcutCallback: (callback: (id: number) => void) => void;
/** payload = "width,height:base64(RGBA|BGRA)". */
export const registerFaviconCallback: (callback: (payload: string) => void) => void;
/** URL of an attachment download observed by the engine. */
export const registerDownloadSink: (callback: (url: string) => void) => void;
export const focusWebview:(index: number, arkts_ids:number[]) => void;
export const deleteWebview:(index: number, arkts_ids:number[]) => void;
export const initServo:(options: InitOpts) => void;
export const nextWindowId:(arkts_id: number) => void;
