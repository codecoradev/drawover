import { writable } from 'svelte/store';
import type { Tool } from './types';

/** Whether draw mode is ON (user can draw on the screen) */
export const drawMode = writable<boolean>(false);

/** Currently active drawing tool */
export const currentTool = writable<Tool>('pen');

/** Currently selected stroke color (hex string) */
export const currentColor = writable<string>('#ef4444');

/** Current stroke thickness in pixels */
export const currentThickness = writable<number>(4);
