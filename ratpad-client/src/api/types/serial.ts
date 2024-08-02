export type UsbPortType = {
    vid: number;
    pid: number;
    serial_number?: string;
    manufacturer?: string;
    product?: string;
};

export type PortType =
    | "PciPort"
    | "BluetoothPort"
    | "Unknown"
    | {
          UsbPort: UsbPortType;
      };

export type PortInfo = {
    port_name: string;
    port_type: PortType;
};

export enum ConnectionState {
    Connected = "Connected",
    Waiting = "Waiting",
    Disconnected = "Disconnected",
}
