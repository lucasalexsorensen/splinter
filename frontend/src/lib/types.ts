export type Message = CountUpdatedMessage | TargetUpdatedMessage | GyroUpdatedMessage;
export type MessageType = Message['type'];
export type CountUpdatedMessage = {
	type: 'count_updated';
	left: number;
	right: number;
};

export type TargetUpdatedMessage = {
	type: 'target_updated';
	left: number;
	right: number;
};

export type GyroUpdatedMessage = {
	type: 'gyro_updated';
	x: number;
	y: number;
	z: number;
};

export type BotSettings = {
	k_p: number;
	k_d: number;
};

export type Command =
	| { type: 'turn_left' }
	| { type: 'turn_right' }
	| { type: 'move_forward' }
	| { type: 'move_backward' }
	| { type: 'debug_motors' }
	| { type: 'configure'; settings: BotSettings };
