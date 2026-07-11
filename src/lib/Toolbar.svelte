<script lang="ts">
	import type { Tool, Point } from './types';
	import { drawMode, currentTool, currentColor, currentThickness } from './stores';
	import { Pen, Highlighter, Eraser, Undo2, Trash2, X } from '@lucide/svelte';

	interface Props {
		onundo: () => void;
		onclear: () => void;
		onexit: () => void;
	}

	let { onundo, onclear, onexit }: Props = $props();

	// --- Draggable toolbar state ---
	// Start centered vertically on the right edge of the screen.
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
		const tbW = 48; // approx toolbar width (incl padding)
		const tbH = 360; // approx toolbar height (conservative)
		position = {
			x: Math.max(16, window.innerWidth - tbW - 24),
			y: Math.max(16, Math.round((window.innerHeight - tbH) / 2))
		};
		positioned = true;
	}

	// Start auto-fade + initial position on mount
	$effect(() => {
		initPosition();
		resetFade();
		return () => {
			if (fadeTimer) clearTimeout(fadeTimer);
		};
	});

	// --- Dragging ---
	function onPointerDown(e: PointerEvent): void {
		// Only start drag from the toolbar background, not from buttons
		if ((e.target as HTMLElement).closest('[data-btn]')) return;
		dragging = true;
		dragOffset = { x: e.clientX - position.x, y: e.clientY - position.y };
		(e.target as HTMLElement).setPointerCapture(e.pointerId);
		resetFade();
	}

	function onPointerMove(e: PointerEvent): void {
		if (!dragging) return;
		position = {
			x: Math.max(0, Math.min(window.innerWidth - 48, e.clientX - dragOffset.x)),
			y: Math.max(0, Math.min(window.innerHeight - 60, e.clientY - dragOffset.y))
		};
	}

	function onPointerUp(e: PointerEvent): void {
		dragging = false;
		(e.target as HTMLElement).releasePointerCapture?.(e.pointerId);
	}

	// --- Tool selection ---
	function selectTool(tool: Tool): void {
		currentTool.set(tool);
		resetFade();
	}

	function selectColor(color: string): void {
		currentColor.set(color);
		// If eraser was selected, switch back to pen when choosing a color
		currentTool.update((t) => (t === 'eraser' ? 'pen' : t));
		resetFade();
	}

	// Auto-subscribe to stores (Svelte 5 $store syntax in template)

	// Shortcut keys shown on hover tooltips. Icon component rendered in template.
	const tools: { id: Tool; icon: typeof Pen; label: string; key: string }[] = [
		{ id: 'pen', icon: Pen, label: 'Pen', key: 'P' },
		{ id: 'highlighter', icon: Highlighter, label: 'Highlighter', key: 'H' },
		{ id: 'eraser', icon: Eraser, label: 'Eraser', key: 'E' }
	];

	const colors = ['#ef4444', '#facc15', '#22c55e', '#3b82f6'];
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
	aria-orientation="vertical"
>
	<!-- Tool buttons -->
	{#each tools as tool (tool.id)}
		{@const Icon = tool.icon}
		<button
			data-btn
			class="btn tool-btn"
			class:active={$currentTool === tool.id}
			onclick={() => selectTool(tool.id)}
			aria-label={tool.label}
			aria-pressed={$currentTool === tool.id}
			title="{tool.label} ({tool.key})"
		>
			<Icon size={18} strokeWidth={2} />
			<span class="keyhint">{tool.key}</span>
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

	<!-- Undo -->
	<button data-btn class="btn" onclick={onundo} aria-label="Undo" title="Undo (⌘Z)">
		<Undo2 size={18} strokeWidth={2} />
		<span class="keyhint">Z</span>
	</button>

	<!-- Clear all -->
	<button data-btn class="btn" onclick={onclear} aria-label="Clear all" title="Clear all (⌘⇧⌫)">
		<Trash2 size={18} strokeWidth={2} />
	</button>

	<!-- Exit -->
	<button data-btn class="btn exit-btn" onclick={onexit} aria-label="Exit draw mode" title="Exit (Esc)">
		<X size={18} strokeWidth={2} />
		<span class="keyhint">Esc</span>
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
		border-radius: 18px;
		background: rgba(30, 30, 36, 0.72);
		backdrop-filter: blur(20px) saturate(180%);
		-webkit-backdrop-filter: blur(20px) saturate(180%);
		border: 1px solid rgba(255, 255, 255, 0.12);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
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
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 34px;
		height: 34px;
		border-radius: 10px;
		border: none;
		background: transparent;
		color: rgba(255, 255, 255, 0.85);
		font-size: 15px;
		cursor: pointer;
		transition: background 0.15s ease, transform 0.1s ease;
		padding: 0;
	}

	.btn:hover {
		background: rgba(255, 255, 255, 0.14);
	}

	.btn:active {
		transform: scale(0.92);
	}

	.tool-btn.active {
		background: rgba(99, 162, 255, 0.3);
		color: #fff;
		box-shadow: inset 0 0 0 1px rgba(99, 162, 255, 0.5);
	}

	/* Keyboard shortcut hint badge, only visible on hover */
	.keyhint {
		position: absolute;
		top: 2px;
		right: 3px;
		font: 600 8px ui-monospace, SFMono-Regular, Menlo, monospace;
		color: rgba(255, 255, 255, 0.45);
		opacity: 0;
		transition: opacity 0.15s ease;
		pointer-events: none;
	}

	.btn:hover .keyhint {
		opacity: 1;
	}

	.color-btn {
		position: relative;
	}

	.color-btn .dot {
		display: block;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 1.5px solid rgba(255, 255, 255, 0.3);
		transition: transform 0.1s ease;
	}

	.color-btn.active .dot {
		transform: scale(1.15);
		border-color: rgba(255, 255, 255, 0.9);
		box-shadow: 0 0 0 2px rgba(99, 162, 255, 0.4);
	}

	.color-btn.active {
		background: rgba(255, 255, 255, 0.1);
	}

	.divider {
		width: 22px;
		height: 1px;
		background: rgba(255, 255, 255, 0.15);
		margin: 2px 0;
		flex-shrink: 0;
	}

	.exit-btn {
		color: rgba(255, 100, 100, 0.9);
	}

	.exit-btn:hover {
		background: rgba(255, 80, 80, 0.2);
		color: #ff6464;
	}
</style>
