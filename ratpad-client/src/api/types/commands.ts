import { invoke } from "@tauri-apps/api";
import { ConnectionState, PortInfo } from "./serial";
import { AppModeConfig } from "./mode";

type CommandType<Type extends `${string}.${string}`, Data> = {
    type: Type;
} & Data;

interface CommandReturnType<Type extends `${string}.${string}`, Value> {
    type: Type;
    result?: Value;
}

type CommandSpec<
    Type extends `${string}.${string}` = any,
    Params = any,
    Return = any
> = {
    command: CommandType<Type, Params>;
    returnType: CommandReturnType<Type, Return>;
};

export type SerialConnect = CommandSpec<
    "serial.connect",
    { port: string; rate: number }
>;

export type SerialDisconnect = CommandSpec<"serial.disconnect">;

export type SerialPortList = CommandSpec<"serial.list_ports", null, PortInfo[]>;

export type SerialConnectionState = CommandSpec<
    "serial.get_state",
    null,
    { connected: ConnectionState; port?: string; rate?: number }
>;

export type GetConfig = CommandSpec<
    "config.get_config",
    {},
    { config: AppModeConfig }
>;

export type SetColorType =
    | { key: "next" | "previous" | "select"; color: [number, number, number] }
    | { key: "brightness"; color: number };

export type ConfSetColor = CommandSpec<"pad.set_color", SetColorType>;

export type ConfWriteMode = CommandSpec<
    "config.write_mode",
    { mode: AppModeConfig }
>;

export type ConfDeleteMode = CommandSpec<"config.delete_mode", { key: string }>;

export type ConfClearModes = CommandSpec<"config.clear_modes">;

export type PadSetHome = CommandSpec<"pad.set_home">;

export type PadSetMode = CommandSpec<"pad.set_mode", { mode: string }>;

export class CommandResult<T extends CommandSpec> {
    public constructor(
        private cmd: T["command"],
        private is_success: boolean,
        private output: T["returnType"]["result"] | string
    ) {}

    public get success(): boolean {
        return this.is_success;
    }

    public get command(): T["command"] {
        return this.cmd;
    }

    public get result(): T["returnType"]["result"] | null {
        return this.success ? this.output : null;
    }

    public get error(): string | null {
        return this.success ? null : this.output;
    }
}

export async function executeCommand<T extends CommandSpec>(
    command: T["command"]
): Promise<CommandResult<T>> {
    try {
        const result = await invoke<T["returnType"]>("execute_command", {
            command,
        });
        return new CommandResult<T>(command, true, result);
    } catch (e) {
        return new CommandResult<T>(command, false, e);
    }
}
