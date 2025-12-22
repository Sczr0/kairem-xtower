<svelte:options runes={false} />

<script lang="ts">
	function reload() {
		if (typeof window === 'undefined') return;
		window.location.reload();
	}
</script>

<svelte:head>
	<meta name="robots" content="noindex,nofollow" />
</svelte:head>

<div class="offline">
	<div class="offline-card">
		<div class="offline-title">你当前处于离线状态</div>
		<div class="offline-sub">
			网络恢复后可重试。若你已安装 PWA，通常仍可继续游玩<strong>已访问过</strong>的页面与题目。
		</div>

		<div class="offline-actions">
			<button class="btn btn-primary" type="button" on:click={reload}>重试</button>
			<a class="btn btn-ghost" href="/">回到首页</a>
		</div>

		<div class="offline-tip">
			提示：离线时无法首次加载新的深链接（例如未访问过的日期/关卡），但你之前打开过的页面可能已在缓存中。
		</div>
	</div>
</div>

<style>
	.offline {
		min-height: min(520px, 70vh);
		display: grid;
		place-items: center;
		padding: 24px 16px;
	}

	.offline-card {
		width: min(680px, 100%);
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-soft);
		padding: 18px 18px;
	}

	.offline-title {
		font-size: 1.25rem;
		font-weight: 950;
		letter-spacing: -0.02em;
		margin-bottom: 6px;
	}

	.offline-sub {
		color: var(--muted);
		line-height: 1.55;
		margin-bottom: 14px;
	}

	.offline-actions {
		display: flex;
		gap: 10px;
		flex-wrap: wrap;
		margin-bottom: 12px;
	}

	.offline-tip {
		color: var(--muted);
		font-size: 0.92rem;
		line-height: 1.55;
	}
</style>

