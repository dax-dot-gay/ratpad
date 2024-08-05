import { AppModeConfig } from "./mode";

export type Color = [number, number, number];

export type AppColorsConfig = {
    next: Color;
    previous: Color;
    select: Color;
    brightness: number;
};

export type AppConfig = {
    device_port: string | null;
    device_rate: number | null;
    colors: AppColorsConfig;
    modes: AppModeConfig[];
};

export enum ConnectionState {
    Connected = "Connected",
    Disconnected = "Disconnected",
    Waiting = "Waiting",
}

export type AppState = {
    connection: ConnectionState;
    config: AppConfig;
};
