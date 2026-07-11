<script lang="ts">
	let { children } = $props();
</script>

<div
	style="position:fixed;inset:0;width:100vw;height:100vh;padding:0;margin:0;overflow:hidden;background:transparent;"
>
	{@render children?.()}
</div>

<style>
	:global(*),
	:global(*::before),
	:global(*::after) {
		box-sizing: border-box;
	}

	:global(html),
	:global(body) {
		margin: 0;
		padding: 0;
		background: transparent;
	}
</style>
