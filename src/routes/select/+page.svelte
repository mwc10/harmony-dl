<script lang="ts">
    import type { WellInfo, XmlInfo } from "$lib/ffi_types";
    import WellPlate from "../WellPlate.svelte";
    import { range } from "$lib/range"
    import { invoke } from "@tauri-apps/api/core";


    let { data }: {data: {info: XmlInfo}} = $props();
    const info: XmlInfo = data.info

    // Channels
    let active_channels = $state(
        info.channels.map((c) => c.name)
    )
    // Wells
    const imaged_wells = (wells: (WellInfo | null)[][]) => {
        return wells.map((r) => r.map((w) => w !== null))
    }
    let active_wells = $state(imaged_wells(info.wells))
    const toggle_well = (r:number, c:number) => {
        active_wells[r][c] = !active_wells[r][c]
    }
    const toggle_row = (r:number) => {
        // if any off well, turn all on
        const current = active_wells[r].some((w) => !w)
        active_wells[r] = active_wells[r].map((_) => current)
    }
    const toggle_col = (c:number) => {
        const current = active_wells
            .map((r) => r[c])
            .some((w) => !w)
        for (let r = 0; r <  active_wells.length; r++) {
            active_wells[r][c] = current
        }
    }
    // Fields
    let field_start = $state(1)
    let field_end = $state(info.fields)
    let field_step = $state(1)

    // Planes
    let plane_low = $state(1)
    let plane_high = $state(info.planes)

    // create summary and send to rust...?
    const apply_filter = async () => {
        const wells = active_wells.flatMap((r, i) => {
            return r.map((w, j) => [w, i, j])
            .filter(([w, ..._]) => w)
            .map(([_, i, j]) => [Number(i)+1, Number(j)+1])

        })
        const filter = {
            channels: [...active_channels],
            wells,
            fields: [...range(field_start, field_end+1, field_step)],
            planes: [...range(plane_low, plane_high+1)],
        }

        await invoke('set_filter', {filter})

        
    }

</script>

<main>
    <h1>Data Selection</h1>
    <h2>{info.name}</h2>
    <h3>Channels</h3>
    {#each info.channels as c}
    <label>
        <input
            type="checkbox"
            name="channels"
            value={c.name}
            bind:group={active_channels}
        />
        {c.name}
    </label>
    {/each}

    <h3> Wells </h3>
    <WellPlate plate={active_wells}>
        {#snippet rowHdr(r:number)}
        <th scope="row" onclick={()=>toggle_row(r)}>
            {String.fromCharCode(65+r)}
        </th>
        {/snippet}
        {#snippet colHdr(c: number)}
        <th scope="col" onclick={()=>toggle_col(c)}>
            {c + 1}
        </th>
        {/snippet}
        {#snippet well(r:number, c:number, chosen: boolean)}
        <td class="{chosen?'active':'inactive'}" onclick={(_) => toggle_well(r,c)}></td>
        {/snippet}
    </WellPlate>

    <h3> Fields </h3>
    <p> {JSON.stringify(info.fields)} Total </p>
    <label for="fStart">First Field: </label>
    <input id="fStart" type="number" bind:value={field_start} />

    <label for="fEnd">Last Field: </label>
    <input id="fEnd" type="number" bind:value={field_end} />

    <label for="fStep">Step by: </label>
    <input id="fStep" type="number" bind:value={field_step} />

    <h3> Planes </h3>
    <label for="pMin">Lowest: </label>
    <input id="pMin" type="number" bind:value={plane_low} />

    <label for="pMax">Highest: </label>
    <input id="pMax" type="number" bind:value={plane_high} />
    <hr />

    <button onclick={apply_filter}>Fetch Images</button>
    
    <hr />
    <a href="/">Select Another File?</a>
</main>

<style>
    th {
        min-width: 1.25rem;
        cursor: pointer;
    }
    td {
        min-width: 2rem;
        cursor: pointer;
    }
    .active {
        background-color: green;
    }
    .inactive {
        background-color: lightslategray;
    }
    label {
        cursor: pointer;
        display: inline-block;
        padding: 0.5rem 1rem;
    }
</style>


