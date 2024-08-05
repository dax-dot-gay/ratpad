export type AppKeyAction =
    | { type: "none" }
    | { type: "keypress"; key: string }
    | { type: "command"; execute: string; args: string[] | null };

export type AppKeyConfig = {
    label: string;
    action: AppKeyAction;
    color: [number, number, number] | null;
};

export type AppModeConfig = {
    key: string;
    title: string;
    title_short: string;
    color: [number, number, number] | null;
    keys: (AppKeyConfig | null)[];
};
