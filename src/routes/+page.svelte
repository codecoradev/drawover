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

	// Store subscription values
	let drawModeOn = $state(false);
	let tool = $state<Tool>('pen');
	let color = $state('#ef4444');
	let thickness = $state(4);

	const unsubDrawMode = drawMode.subscribe((v) => (drawModeOn = v));
	const unsubTool = currentTool.subscribe((v) => (tool = v));
	const unsubColor = currentColor.subscribe((v) => (color = v));
	const unsubThickness = currentThickness.subscribe((v) => (thickness = v));

	// ─── Helpers ───
	const SHAPE_TOOLS: Tool[] = ['line', 'arrow', 'rectangle', 'ellipse'];

	function isShape(t: Tool): boolean {
		return SHAPE_TOOLS.includes(t);
	}

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

		const pt = getPoint(e);
		if (isShape(tool)) {
			// For shapes, we only keep start + current point
			currentPoints = [currentPoints[0], pt];
		} else {
			currentPoints.push(pt);
		}
		renderIncremental();
	}

	function onPointerUp(e: PointerEvent): void {
		if (!isDrawing) return;
		e.preventDefault();

		isDrawing = false;
		canvas.releasePointerCapture?.(e.pointerId);

		if (currentPoints.length < 2) {
			// Dot for pen/highlighter; ignore for shapes
			if (isShape(tool)) {
				currentPoints = [];
				return;
			}
			const p = currentPoints[0];
			currentPoints = [
				{ x: p.x - 0.5, y: p.y },
				{ x: p.x + 0.5, y: p.y }
			];
		}

		const stroke: Stroke = {
			id: crypto.randomUUID(),
			points: [...currentPoints],
			color: color,
			width: tool === 'eraser' ? 30 : tool === 'highlighter' ? thickness * 4 : thickness,
			tool: tool,
			opacity: tool === 'highlighter' ? 0.4 : 1
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

	function drawArrowHead(fromX: number, fromY: number, toX: number, toY: number): void {
		const headLen = Math.max(12, ctx.lineWidth * 2.5);
		const angle = Math.atan2(toY - fromY, toX - fromX);
		ctx.beginPath();
		ctx.moveTo(toX, toY);
		ctx.lineTo(
			toX - headLen * Math.cos(angle - Math.PI / 6),
			toY - headLen * Math.sin(angle - Math.PI / 6)
		);
		ctx.moveTo(toX, toY);
		ctx.lineTo(
			toX - headLen * Math.cos(angle + Math.PI / 6),
			toY - headLen * Math.sin(angle + Math.PI / 6)
		);
		ctx.stroke();
	}

	function drawShape(s: Stroke): void {
		if (s.points.length < 2) return;
		const [a, b] = [s.points[0], s.points[s.points.length - 1]];

		ctx.beginPath();
		if (s.tool === 'line') {
			ctx.moveTo(a.x, a.y);
			ctx.lineTo(b.x, b.y);
			ctx.stroke();
		} else if (s.tool === 'arrow') {
			ctx.moveTo(a.x, a.y);
			ctx.lineTo(b.x, b.y);
			ctx.stroke();
			drawArrowHead(a.x, a.y, b.x, b.y);
		} else if (s.tool === 'rectangle') {
			ctx.strokeRect(a.x, a.y, b.x - a.x, b.y - a.y);
		} else if (s.tool === 'ellipse') {
			const cx = (a.x + b.x) / 2;
			const cy = (a.y + b.y) / 2;
			const rx = Math.abs(b.x - a.x) / 2;
			const ry = Math.abs(b.y - a.y) / 2;
			ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
			ctx.stroke();
		}
	}

	function drawStroke(s: Stroke): void {
		if (isShape(s.tool)) {
			ctx.save();
			setStrokeStyle(s);
			drawShape(s);
			ctx.restore();
			return;
		}

		if (s.points.length < 2) return;

		ctx.save();
		setStrokeStyle(s);

		const pts = s.points;
		if (pts.length === 2) {
			ctx.beginPath();
			ctx.moveTo(pts[0].x, pts[0].y);
			ctx.lineTo(pts[1].x, pts[1].y);
			ctx.stroke();
		} else {
			// Catmull-Rom spline interpolation
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

	/** Render the in-progress stroke (freehand or shape preview) */
	function renderIncremental(): void {
		redraw();
		if (currentPoints.length < 2) return;

		ctx.save();
		const isHl = tool === 'highlighter';
		const isEr = tool === 'eraser';
		ctx.globalCompositeOperation = isEr ? 'destination-out' : isHl ? 'multiply' : 'source-over';
		ctx.globalAlpha = isHl ? 0.4 : 1;
		ctx.strokeStyle = color;
		ctx.lineWidth = isEr ? 30 : isHl ? thickness * 4 : thickness;
		ctx.lineCap = 'round';
		ctx.lineJoin = 'round';

		const pts = currentPoints;

		if (isShape(tool)) {
			// Shape preview — draw from the preview stroke
			const previewStroke: Stroke = {
				id: 'preview',
				points: pts,
				color,
				width: thickness,
				tool,
				opacity: 1
			};
			drawShape(previewStroke);
		} else if (pts.length === 2) {
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

	// ─── Keyboard shortcuts ───
	const TOOL_KEYS: Record<string, Tool> = {
		p: 'pen',
		h: 'highlighter',
		l: 'line',
		a: 'arrow',
		r: 'rectangle',
		o: 'ellipse',
		e: 'eraser'
	};

	function handleKeydown(e: KeyboardEvent): void {
		if (!drawModeOn) return;

		// Undo
		if ((e.metaKey || e.ctrlKey) && e.key === 'z' && !e.shiftKey) {
			e.preventDefault();
			undo();
			return;
		}

		// Clear all
		if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
			e.preventDefault();
			clearAll();
			return;
		}

		// Brush size
		if (e.key === '=' || e.key === '+') {
			e.preventDefault();
			currentThickness.update((t) => Math.min(20, t + 2));
			return;
		}
		if (e.key === '-' || e.key === '_') {
			e.preventDefault();
			currentThickness.update((t) => Math.max(2, t - 2));
			return;
		}

		// Tool selection
		const k = e.key.toLowerCase();
		if (TOOL_KEYS[k] && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			currentTool.set(TOOL_KEYS[k]);
		}

		// Color shortcuts (1-7)
		const colorMap = ['#ef4444', '#f97316', '#facc15', '#22c55e', '#3b82f6', '#a855f7', '#ffffff'];
		const numKey = parseInt(e.key);
		if (numKey >= 1 && numKey <= 7 && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			currentColor.set(colorMap[numKey - 1]);
		}
	}

	// ─── Tauri event listeners ───

	onMount(() => {
		setupCanvas();
		window.addEventListener('resize', onResize);
		window.addEventListener('keydown', handleKeydown);

		let unlistenDrawToggle: UnlistenFn | null = null;

		listen<boolean>('draw-mode-toggled', (event) => {
			drawMode.set(event.payload);
		})
			.then((un) => {
				unlistenDrawToggle = un;
			})
			.catch(() => {});

		invoke<boolean>('is_draw_mode')
			.then((v) => {
				drawMode.set(v);
			})
			.catch(() => {});

		return () => {
			window.removeEventListener('resize', onResize);
			window.removeEventListener('keydown', handleKeydown);
			unsubDrawMode();
			unsubTool();
			unsubColor();
			unsubThickness();
			unlistenDrawToggle?.();
		};
	});

	// ─── Redraw when state changes ───
	$effect(() => {
		if (drawModeOn !== undefined) {
			redraw();
		}
	});
</script>

<svelte:window onresize={onResize} />

<main class:draw-mode={drawModeOn}>
	{#if drawModeOn}
		<button
			onclick={toggleDrawMode}
			class="fab-stop"
			title="Stop drawing (Alt+Shift+D)"
			aria-label="Stop drawing"
		>
			✕
		</button>
	{/if}

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

	.fab-stop {
		position: fixed;
		bottom: 24px;
		left: 24px;
		z-index: 99999;
		width: 48px;
		height: 48px;
		border-radius: 50%;
		border: none;
		background: rgba(239, 68, 68, 0.9);
		color: white;
		font-size: 20px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
		transition: transform 0.15s ease, background 0.15s ease;
		-webkit-user-select: none;
		user-select: none;
	}

	.fab-stop:hover {
		background: rgba(220, 38, 38, 0.95);
		transform: scale(1.08);
	}

	.fab-stop:active {
		transform: scale(0.95);
	}
</style>
