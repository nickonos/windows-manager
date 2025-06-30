<script lang="ts">
	import { Button } from "$lib/components/ui/button/index";
	import * as Table from "$lib/components/ui/table/index";
	import Plus from "@lucide/svelte/icons/plus";

	const entries = [
		{
			application: "Teams",
			shortcut: ["windows", "shift", "t"],
			enabled: true,
			admin: false,
		},
		{
			application: "Teams",
			shortcut: ["windows", "shift", "t"],
			enabled: true,
			admin: false,
		},
		{
			application: "Teams",
			shortcut: ["windows", "shift", "t"],
			enabled: true,
			admin: false,
		},
	];
</script>

<main class="p-4 flex bg-background min-h-screen">
	<Button class="absolute bottom-4 right-4 rounded-2xl w-14 h-14" size="icon">
		<Plus class="stroke-2 size-8" />
	</Button>
	<Table.Root>
		<Table.Caption>All registered shortcuts</Table.Caption>
		<Table.Header>
			<Table.Row>
				<Table.Head class="w-[100px]">Application</Table.Head>
				<Table.Head>Shortcut</Table.Head>
				<Table.Head>Enabled</Table.Head>
				<Table.Head>Admin</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#each entries as entry, index (entry)}
				<Table.Row class={index %2 == 1 ? "bg-card" : ""}>
					<Table.Cell class="w-[100px]"
						>{entry.application}</Table.Cell
					>
					<Table.Cell>{entry.shortcut.join("+")}</Table.Cell>
					<Table.Cell>{entry.enabled}</Table.Cell>
					<Table.Cell>{entry.admin}</Table.Cell>
				</Table.Row>
			{/each}
		</Table.Body>
	</Table.Root>
</main>
