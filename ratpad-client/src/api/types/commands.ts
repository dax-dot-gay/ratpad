import { invoke } from "@tauri-apps/api";

interface CommandType<Type extends `${string}.${string}`, Data> {
    type: Type;
    data: Data | null;
}

interface CommandReturnType<Type extends `${string}.${string}`, Value> {
    type: Type;
    value: Value | null;
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

export type SerialPortList = CommandSpec<"serial.list_ports", null, any>;

export async function executeCommand<T extends CommandSpec>(
    command: T["command"]
): Promise<T["returnType"]["value"] | string> {
    try {
        const result = await invoke<T["returnType"]>("execute_command", {
            command,
        });
        return result;
    } catch (e) {
        return e;
    }
}
