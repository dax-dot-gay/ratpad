import { AppConfig, AppState } from "./types/app";

export type ApiContextType = {
    state: AppState | null;
    updateState: () => Promise<AppConfig | null>;
};
