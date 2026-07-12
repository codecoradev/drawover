<script lang="ts">
	import type { Tool, Point } from './types';
	import { drawMode, currentTool, currentColor, currentThickness } from './stores';

	interface Props {
		onundo: () => void;
		onclear: () => void;
		onexit: () => void;
	}

	let { onundo, onclear, onexit }: Props = $props();

	// --- Draggable toolbar state ---
	let position = $state<Point>({ x: 0, y: 0 });
	let positioned = $state(false);
	let dragging = $state(false);
	let dragOffset: Point = { x: 0, y: 0 };

	// --- Auto-fade state ---
	let visible = $state(true);
	let fadeTimer: ReturnType<typeof setTimeout> | null = null;

	const FADE_DELAY = 3000;

	function resetFade(): void {
		visible = true;
		if (fadeTimer) clearTimeout(fadeTimer);
		fadeTimer = setTimeout(() => {
			visible = false;
		}, FADE_DELAY);
	}

	// Place toolbar at right-center on first layout.
	function initPosition(): void {
		if (positioned) return;
		const tbW = 52;
		const tbH = 520;
		position = {
			x: Math.max(16, window.innerWidth - tbW - 24),
			y: Math.max(16, Math.round((window.innerHeight - tbH) / 2))
		};
		positioned = true;
	}

	$effect(() => {
		initPosition();
		resetFade();
		return () => {
			if (fadeTimer) clearTimeout(fadeTimer);
		};
	});

	// --- Dragging ---
	function onPointerDown(e: PointerEvent): void {
		if ((e.target as HTMLElement).closest('[data-btn]')) return;
		dragging = true;
		dragOffset = { x: e.clientX - position.x, y: e.clientY - position.y };
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
		resetFade();
	}

	function onPointerMove(e: PointerEvent): void {
		if (!dragging) return;
		position = {
			x: Math.max(0, Math.min(window.innerWidth - 52, e.clientX - dragOffset.x)),
			y: Math.max(0, Math.min(window.innerHeight - 60, e.clientY - dragOffset.y))
		};
	}

	function onPointerUp(e: PointerEvent): void {
		dragging = false;
		(e.target as HTMLElement).releasePointerCapture?.(e.pointerId);
	}

	// --- Tool selection ---
	function selectTool(t: Tool): void {
		currentTool.set(t);
		resetFade();
	}

	function selectColor(color: string): void {
		currentColor.set(color);
		currentTool.update((t) => (t === 'eraser' ? 'pen' : t));
		resetFade();
	}

	// --- Thickness slider ---
	function changeThickness(delta: number): void {
		currentThickness.update((t) => Math.max(2, Math.min(20, t + delta)));
		resetFade();
	}

	const tools: { id: Tool; icon: string; label: string; key: string }[] = [
		{ id: 'pen', icon: '✏️', label: 'Pen', key: 'P' },
		{ id: 'highlighter', icon: '🖍️', label: 'Highlighter', key: 'H' },
		{ id: 'line', icon: '╱', label: 'Line', key: 'L' },
		{ id: 'arrow', icon: '→', label: 'Arrow', key: 'A' },
		{ id: 'rectangle', icon: '▭', label: 'Rectangle', key: 'R' },
		{ id: 'ellipse', icon: '◯', label: 'Ellipse', key: 'O' },
		{ id: 'eraser', icon: '⌫', label: 'Eraser', key: 'E' }
	];

	const colors = ['#ef4444', '#f97316', '#facc15', '#22c55e', '#3b82f6', '#a855f7', '#ffffff'];
</script>

<svelte:window onpointermove={onPointerMove} />

<div
	class="toolbar"
	class:visible
	class:dragging
	style="left: {position.x}px; top: {position.y}px;"
	tabindex="-1"
	onpointerdown={onPointerDown}
	onpointerup={onPointerUp}
	onpointerenter={resetFade}
	role="toolbar"
	aria-label="DrawOver toolbar"
>
	<!-- Drawing tools -->
	{#each tools as t (t.id)}
		<button
			data-btn
			class="btn tool-btn"
			class:active={$currentTool === t.id}
			onclick={() => selectTool(t.id)}
			aria-label={t.label}
			aria-pressed={$currentTool === t.id}
			title="{t.label} ({t.key})"
		>
			{t.icon}
		</button>
	{/each}

	<!-- Divider -->
	<div class="divider"></div>

	<!-- Color presets -->
	{#each colors as c (c)}
		<button
			data-btn
			class="btn color-btn"
			class:active={$currentColor === c}
			style="--color: {c};"
			onclick={() => selectColor(c)}
			aria-label="Color {c}"
			aria-pressed={$currentColor === c}
			title="Color {c}"
		>
			<span class="dot" style="background: {c};"></span>
		</button>
	{/each}

	<!-- Divider -->
	<div class="divider"></div>

	<!-- Brush size controls -->
	<div class="size-row">
		<button data-btn class="btn size-btn" onclick={() => changeThickness(-2)} aria-label="Thinner" title="Thinner (−)">
			−
		</button>
		<span class="size-display">{$currentThickness}</span>
		<button data-btn class="btn size-btn" onclick={() => changeThickness(2)} aria-label="Thicker" title="Thicker (+)">
			+
		</button>
	</div>

	<!-- Divider -->
	<div class="divider"></div>

	<!-- Undo -->
	<button data-btn class="btn" onclick={onundo} aria-label="Undo" title="Undo (⌘Z)">
		↩
	</button>

	<!-- Clear all -->
	<button data-btn class="btn" onclick={onclear} aria-label="Clear all" title="Clear all (⌘K)">
		🗑
	</button>

	<!-- Exit -->
	<button data-btn class="btn exit-btn" onclick={onexit} aria-label="Exit draw mode" title="Exit (Alt+Shift+D)">
		✕
	</button>
</div>

<style>
	.toolbar {
		position: fixed;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 8px 6px;
		border-radius: 16px;
		background: rgba(26, 26, 31, 0.82);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		border: 1px solid rgba(255, 255, 255, 0.1);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
		z-index: 9999;
		opacity: 0.25;
		transition: opacity 0.3s ease;
		-webkit-user-select: none;
		user-select: none;
		cursor: grab;
	}

	.toolbar.visible {
		opacity: 1;
	}

	.toolbar.dragging {
		cursor: grabbing;
		opacity: 1;
		transition: none;
	}

	.btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 10px;
		border: none;
		background: transparent;
		color: rgba(255, 255, 255, 0.8);
		font-size: 16px;
		cursor: pointer;
		transition: background 0.15s ease, transform 0.1s ease;
		padding: 0;
	}

	.btn:hover {
		background: rgba(255, 255, 255, 0.12);
	}

	.btn:active {
		transform: scale(0.92);
	}

	.tool-btn.active {
		background: rgba(239, 89, 63, 0.25);
		color: #fff;
		box-shadow: inset 0 0 0 1px rgba(239, 89, 63, 0.5);
	}

	.color-btn {
		position: relative;
	}

	.color-btn .dot {
		display: block;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 1.5px solid rgba(255, 255, 255, 0.3);
		transition: transform 0.1s ease;
	}

	.color-btn.active .dot {
		transform: scale(1.15);
		border-color: rgba(255, 255, 255, 0.9);
		box-shadow: 0 0 0 2px rgba(239, 89, 63, 0.4);
	}

	.color-btn.active {
		background: rgba(255, 255, 255, 0.08);
	}

	.divider {
		width: 28px;
		height: 1px;
		background: rgba(255, 255, 255, 0.12);
		margin: 2px 0;
		flex-shrink: 0;
	}

	.size-row {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.size-btn {
		width: 28px;
		height: 28px;
		font-size: 14px;
		font-weight: 600;
	}

	.size-display {
		font-size: 12px;
		font-family: monospace;
		color: rgba(255, 255, 255, 0.6);
		min-width: 20px;
		text-align: center;
	}

	.exit-btn {
		color: rgba(255, 100, 100, 0.9);
	}

	.exit-btn:hover {
		background: rgba(255, 80, 80, 0.2);
		color: #ff6464;
	}
</style>
