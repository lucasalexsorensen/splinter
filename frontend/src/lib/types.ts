export type ConnectionState = 'connected' | 'disconnected' | 'connecting' | 'error';

export type BotSettings = {
	k_p: number;
	k_d: number;
};

export enum Command {
	TurnLeft,
	TurnRight,
	MoveForward,
	MoveBackward,
	DebugMotors,
	Configure
}
