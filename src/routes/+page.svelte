<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import Toolbar from '$lib/Toolbar.svelte';
	import { drawMode, currentTool, currentColor, currentThickness } from '$lib/stores';
	import type { Point, Stroke, Tool } from '$lib/types';

	// ─── Canvas refs ───
	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D;

	// ─── Drawing state ───
	let strokes = $state<Stroke[]>([]);
	let isDrawing = $state(false);
	let currentPoints: Point[] = [];
	let dpr = 1;
	let pdown = $state(0);

	// Store subscription values for template reactivity
	let drawModeOn = $state(false);
	let tool = $state<Tool>('pen');
	let color = $state('#ef4444');
	let thickness = $state(4);

	const unsubDrawMode = drawMode.subscribe((v) => (drawModeOn = v));
	const unsubTool = currentTool.subscribe((v) => (tool = v));
	const unsubColor = currentColor.subscribe((v) => (color = v));
	const unsubThickness = currentThickness.subscribe((v) => (thickness = v));

	// ─── Canvas setup ───
	function setupCanvas(): void {
		if (!canvas) return;
		dpr = window.devicePixelRatio || 1;
		canvas.width = window.innerWidth * dpr;
		canvas.height = window.innerHeight * dpr;
		canvas.style.width = `${window.innerWidth}px`;
		canvas.style.height = `${window.innerHeight}px`;
		ctx = canvas.getContext('2d')!;
		ctx.scale(dpr, dpr);
		ctx.lineCap = 'round';
		ctx.lineJoin = 'round';
		redraw();
	}

	function onResize(): void {
		setupCanvas();
	}

	// ─── Drawing ───
	function getPoint(e: PointerEvent): Point {
		return { x: e.clientX, y: e.clientY };
	}

	function onPointerDown(e: PointerEvent): void {
		pdown++;
		console.log('[DrawOver] pointerdown', { drawModeOn, button: e.button, type: e.pointerType });
		if (!drawModeOn) return;
		if (e.button !== 0 && e.pointerType === 'mouse') return;
		e.preventDefault();

		isDrawing = true;
		currentPoints = [getPoint(e)];
		canvas.setPointerCapture(e.pointerId);
	}

	function onPointerMove(e: PointerEvent): void {
		if (!isDrawing) return;
		e.preventDefault();

		currentPoints.push(getPoint(e));
		// Draw incrementally: render the last segment(s) for performance
		renderIncremental();
	}

	function onPointerUp(e: PointerEvent): void {
		if (!isDrawing) return;
		e.preventDefault();

		isDrawing = false;
		canvas.releasePointerCapture?.(e.pointerId);

		if (currentPoints.length < 2) {
			// Treat as a dot — create a minimal stroke
			const p = currentPoints[0];
			currentPoints = [
				{ x: p.x - 0.5, y: p.y },
				{ x: p.x + 0.5, y: p.y }
			];
		}

		const stroke: Stroke = {
			id: crypto.randomUUID(),
			points: [...currentPoints],
			color: tool === 'eraser' ? '#000000' : color,
			width: tool === 'eraser' ? 30 : tool === 'highlighter' ? thickness * 4 : thickness,
			tool: tool,
			opacity: tool === 'highlighter' ? 0.4 : tool === 'eraser' ? 1 : 1
		};

		strokes = [...strokes, stroke];
		currentPoints = [];
		redraw();
	}

	// ─── Rendering ───
	function setStrokeStyle(s: Stroke): void {
		ctx.globalCompositeOperation = s.tool === 'eraser' ? 'destination-out' : 'source-over';
		ctx.globalAlpha = s.opacity;
		ctx.strokeStyle = s.color;
		ctx.lineWidth = s.width;

		if (s.tool === 'highlighter') {
			ctx.globalCompositeOperation = 'multiply';
		}
	}

	function drawStroke(s: Stroke): void {
		if (s.points.length < 2) return;

		ctx.save();
		setStrokeStyle(s);

		const pts = s.points;
		if (pts.length === 2) {
			// Straight line for 2-point stroke
			ctx.beginPath();
			ctx.moveTo(pts[0].x, pts[0].y);
			ctx.lineTo(pts[1].x, pts[1].y);
			ctx.stroke();
		} else {
			// Catmull-Rom spline interpolation for smooth lines
			ctx.beginPath();
			ctx.moveTo(pts[0].x, pts[0].y);

			for (let i = 0; i < pts.length - 1; i++) {
				const p0 = pts[i - 1] || pts[i];
				const p1 = pts[i];
				const p2 = pts[i + 1];
				const p3 = pts[i + 2] || p2;

				// Catmull-Rom to Bezier conversion (tension = 0.5)
				const cp1x = p1.x + (p2.x - p0.x) / 6;
				const cp1y = p1.y + (p2.y - p0.y) / 6;
				const cp2x = p2.x - (p3.x - p1.x) / 6;
				const cp2y = p2.y - (p3.y - p1.y) / 6;

				ctx.bezierCurveTo(cp1x, cp1y, cp2x, cp2y, p2.x, p2.y);
			}
			ctx.stroke();
		}

		ctx.restore();
	}

	/**
	 * Render the in-progress stroke using Catmull-Rom for the committed points
	 * plus a direct line to the current cursor position for immediate feedback.
	 */
	function renderIncremental(): void {
		redraw();

		if (currentPoints.length < 2) return;

		ctx.save();
		// Apply current tool style to the in-progress stroke
		const isHighlighter = tool === 'highlighter';
		const isEraser = tool === 'eraser';

		ctx.globalCompositeOperation = isEraser ? 'destination-out' : isHighlighter ? 'multiply' : 'source-over';
		ctx.globalAlpha = isHighlighter ? 0.4 : 1;
		ctx.strokeStyle = color;
		ctx.lineWidth = isEraser ? 30 : isHighlighter ? thickness * 4 : thickness;
		ctx.lineCap = 'round';
		ctx.lineJoin = 'round';

		const pts = currentPoints;

		if (pts.length === 2) {
			ctx.beginPath();
			ctx.moveTo(pts[0].x, pts[0].y);
			ctx.lineTo(pts[1].x, pts[1].y);
			ctx.stroke();
		} else {
			ctx.beginPath();
			ctx.moveTo(pts[0].x, pts[0].y);

			for (let i = 0; i < pts.length - 1; i++) {
				const p0 = pts[i - 1] || pts[i];
				const p1 = pts[i];
				const p2 = pts[i + 1];
				const p3 = pts[i + 2] || p2;

				const cp1x = p1.x + (p2.x - p0.x) / 6;
				const cp1y = p1.y + (p2.y - p0.y) / 6;
				const cp2x = p2.x - (p3.x - p1.x) / 6;
				const cp2y = p2.y - (p3.y - p1.y) / 6;

				ctx.bezierCurveTo(cp1x, cp1y, cp2x, cp2y, p2.x, p2.y);
			}
			ctx.stroke();
		}

		ctx.restore();
	}

	function redraw(): void {
		if (!ctx) return;
		ctx.clearRect(0, 0, canvas.width / dpr, canvas.height / dpr);
		for (const s of strokes) {
			drawStroke(s);
		}
	}

	// ─── Actions ───
	async function toggleDrawMode(): Promise<void> {
		try {
			const result = await invoke<boolean>('toggle_draw_mode');
			console.log('[DrawOver] toggle_draw_mode result:', result);
			drawMode.set(result);
		} catch (e) {
			console.error('[DrawOver] toggle_draw_mode failed:', e);
		}
	}

	function undo(): void {
		if (strokes.length === 0) return;
		strokes = strokes.slice(0, -1);
		redraw();
	}

	function clearAll(): void {
		strokes = [];
		redraw();
	}

	async function exitDrawMode(): Promise<void> {
		drawMode.set(false);
		try {
			await invoke('toggle_draw_mode');
		} catch {
			// Tauri command may not exist in dev — safe to ignore
		}
	}

	// ─── Tauri event listeners ───

	onMount(() => {
		setupCanvas();
		window.addEventListener('resize', onResize);

		// Listen for draw mode toggle from global hotkey (Rust side)
		let unlistenDrawToggle: UnlistenFn | null = null;

		listen<boolean>('draw-mode-toggled', (event) => {
			console.log('[DrawOver] draw-mode-toggled event:', event.payload);
			drawMode.set(event.payload);
		})
			.then((un) => {
				unlistenDrawToggle = un;
			})
			.catch(() => {
				// Not in Tauri context (e.g. browser dev) — ignore
			});

		// Sync initial state from Rust backend
		invoke<boolean>('is_draw_mode')
			.then((v) => {
				console.log('[DrawOver] initial is_draw_mode:', v);
				drawMode.set(v);
			})
			.catch(() => {});

		// Clean up on destroy
		return () => {
			window.removeEventListener('resize', onResize);
			unsubDrawMode();
			unsubTool();
			unsubColor();
			unsubThickness();
			unlistenDrawToggle?.();
		};
	});

	// ─── Redraw when draw mode toggles ───
	$effect(() => {
		// Re-render when strokes change or draw mode changes
		if (drawModeOn !== undefined) {
			redraw();
		}
	});
</script>

<svelte:window onresize={onResize} />

<main class:draw-mode={drawModeOn}>
	<div class="debug" style="position:fixed;top:50px;left:8px;z-index:99999;font:12px monospace;background:rgba(0,0,0,0.7);color:#0f0;padding:4px 8px;border-radius:4px;pointer-events:none;">
		mode:{drawModeOn} tool:{tool} cls:{drawModeOn ? 'CAPTURE' : 'pass'} pdown:{pdown} strokes:{strokes.length}
	</div>

	<button
		onclick={toggleDrawMode}
		style="position:fixed;top:8px;right:8px;z-index:99999;padding:8px 14px;background:#3b82f6;color:white;border:none;border-radius:8px;font:14px sans-serif;cursor:pointer;"
	>
		{drawModeOn ? '🔴 STOP Draw' : '✏️ Start Draw'}
	</button>

	<canvas
		bind:this={canvas}
		class:capture={drawModeOn}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerUp}
	></canvas>

	{#if drawModeOn}
		<Toolbar onundo={undo} onclear={clearAll} onexit={exitDrawMode} />
	{/if}
</main>

<style>
	main {
		position: fixed;
		inset: 0;
		width: 100vw;
		height: 100vh;
		overflow: hidden;
		background: transparent;
	}

	canvas {
		position: absolute;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
		pointer-events: none;
		background: transparent;
		touch-action: none;
	}

	canvas.capture {
		pointer-events: auto;
		cursor: crosshair;
	}
</style>
