<svelte:options runes={false} />

<script lang="ts">
	import { onMount } from 'svelte';

	type Theme = 'light' | 'dark';

	let theme: Theme = 'light';

	function readThemeFromDom(): Theme {
		const t = document.documentElement?.dataset?.theme;
		return t === 'dark' ? 'dark' : 'light';
	}

	function setTheme(next: Theme) {
		theme = next;
		document.documentElement.dataset.theme = next;
		try {
			localStorage.setItem('theme', next);
		} catch {
			// 忽略：无痕/隐私模式可能禁用 localStorage
		}
	}

	function toggleTheme() {
		setTheme(theme === 'dark' ? 'light' : 'dark');
	}

	onMount(() => {
		// 与 app.html 的初始化脚本对齐，避免首屏闪烁后再“跳主题”
		theme = readThemeFromDom();
	});
</script>

<button
	type="button"
	class="theme-toggle btn"
	on:click={toggleTheme}
	aria-label={theme === 'dark' ? '切换到浅色主题' : '切换到深色主题'}
	title={theme === 'dark' ? '切换到浅色主题' : '切换到深色主题'}
>
	{#if theme === 'dark'}
		<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
			<path d="M12 18a6 6 0 1 0 0-12a6 6 0 0 0 0 12Z" />
			<path d="M12 2v2" />
			<path d="M12 20v2" />
			<path d="M4.93 4.93l1.41 1.41" />
			<path d="M17.66 17.66l1.41 1.41" />
			<path d="M2 12h2" />
			<path d="M20 12h2" />
			<path d="M4.93 19.07l1.41-1.41" />
			<path d="M17.66 6.34l1.41-1.41" />
		</svg>
		<span class="label">浅色</span>
	{:else}
		<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
			<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z" />
		</svg>
		<span class="label">深色</span>
	{/if}
</button>

<style>
	.theme-toggle {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 999px;
	}

	.icon {
		width: 16px;
		height: 16px;
	}

	.label {
		font-size: 0.86rem;
		color: var(--muted);
	}

	@media (max-width: 520px) {
		.label {
			display: none;
		}
	}
</style>

