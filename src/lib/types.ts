export type Tool = 'pen' | 'highlighter' | 'eraser';

export interface Point {
	x: number;
	y: number;
}

export interface Stroke {
	id: string;
	points: Point[];
	color: string;
	width: number;
	tool: Tool;
	opacity: number;
}

export interface FeatureGate {
	is_pro_activated: boolean;
	is_pro: () => boolean;
}
