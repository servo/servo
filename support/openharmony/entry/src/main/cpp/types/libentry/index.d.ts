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
export const focusWebview:(index: number) => void;
export const initServo:(options: InitOpts) => void;
