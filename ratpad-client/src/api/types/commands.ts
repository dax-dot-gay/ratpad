import { invoke } from "@tauri-apps/api";
import { ConnectionState, PortInfo } from "./serial";

interface CommandType<Type extends `${string}.${string}`, Data> {
    type: Type;
    data: Data | null;
}

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
    { port: string; rate: number },
    null
>;

export type SerialDisconnect = CommandSpec<"serial.disconnect">;

export type SerialPortList = CommandSpec<"serial.list_ports", null, PortInfo[]>;

export type SerialConnectionState = CommandSpec<
    "serial.get_state",
    null,
    { connected: ConnectionState; port?: string; rate?: number }
>;

export type PadGetConfig = CommandSpec<
    "pad.get_config",
    null,
    { config: object }
>;

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
        return new CommandResult<T>(command, true, result.result);
    } catch (e) {
        return new CommandResult<T>(command, false, e);
    }
}
