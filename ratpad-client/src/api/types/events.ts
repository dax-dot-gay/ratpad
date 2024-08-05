export type EventType = "config" | "connect" | "disconnect" | "event" | "log";

export type SerialEvent =
    | { type: "connect" }
    | { type: "disconnect" }
    | { type: "event"; event_type: EventType; data: any };
