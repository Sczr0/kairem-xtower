<svelte:options runes={false} />

<script context="module" lang="ts">
	export type Rule = {
		id: string;
		name: string;
		appliesWhen: string;
		description: string;
	};
</script>

<script lang="ts">
	export let rule: Rule;
	export let color = '#94a3b8';
	export let highlighted = false;

	const appliesWhenLabelMap: Record<string, string> = {
		always: '约束',
		checkedOnly: '仅勾选时',
		goal: '目标'
	};

	// 统一对外文案：避免把内部字段（always/checkedOnly/goal）直接展示给用户。
	$: appliesWhenLabel = appliesWhenLabelMap[rule.appliesWhen] ?? '规则';
	$: appliesWhenClass =
		rule.appliesWhen === 'goal' ? 'goal' : rule.appliesWhen === 'checkedOnly' ? 'checked' : 'always';
</script>

<div class="rule-card {highlighted ? 'highlighted' : ''}">
	<div class="swatch" style="--swatch: {color}" aria-hidden="true"></div>
	<div class="content">
		<div class="header">
			<span class="name">{rule.name}</span>
			<span class="badge {appliesWhenClass}">{appliesWhenLabel}</span>
		</div>
		<div class="desc">{rule.description}</div>
	</div>
</div>

<style>
	.rule-card {
		display: flex;
		gap: 12px;
		padding: 12px;
		border-radius: 12px;
		border: 1px solid rgba(148, 163, 184, 0.25);
		background: rgba(248, 250, 252, 0.9);
		transition:
			border-color 150ms ease,
			box-shadow 150ms ease,
			transform 150ms ease,
			background-color 150ms ease;
	}

	.rule-card.highlighted {
		background: #ffffff;
		border-color: rgba(15, 23, 42, 0.25);
		box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.08), 0 4px 6px -4px rgb(0 0 0 / 0.08);
		transform: translateY(-1px);
	}

	.swatch {
		width: 12px;
		height: 12px;
		border-radius: 4px;
		background: var(--swatch);
		box-shadow: 0 0 0 3px rgba(148, 163, 184, 0.16);
		margin-top: 4px;
		flex-shrink: 0;
	}

	.content {
		min-width: 0;
		flex: 1;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		margin-bottom: 4px;
	}

	.name {
		font-weight: 750;
		font-size: 0.9rem;
		letter-spacing: -0.01em;
		color: #0f172a;
	}

	.badge {
		font-size: 0.65rem;
		padding: 2px 8px;
		border-radius: 999px;
		font-weight: 700;
		letter-spacing: 0.02em;
		white-space: nowrap;
	}

	.badge.always {
		background: rgba(148, 163, 184, 0.18);
		color: #334155;
		border: 1px solid rgba(148, 163, 184, 0.25);
	}

	.badge.checked {
		background: rgba(2, 132, 199, 0.12);
		color: #0369a1;
		border: 1px solid rgba(2, 132, 199, 0.18);
	}

	.badge.goal {
		background: rgba(245, 158, 11, 0.16);
		color: #92400e;
		border: 1px solid rgba(245, 158, 11, 0.22);
	}

	.desc {
		font-size: 0.82rem;
		color: #64748b;
		line-height: 1.45;
	}
</style>
