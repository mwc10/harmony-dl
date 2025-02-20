<script lang="ts">
    import type { WellInfo, XmlInfo } from "$lib/ffi_types";
    import WellPlate from "../WellPlate.svelte";
    import { range } from "$lib/range"
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";


    let { data }: {data: {info: XmlInfo}} = $props();
    const info: XmlInfo = data.info

    // Channels
    let active_channels = $state(
        info.channels.map((c) => c.name)
    )
    // Wells
    type WellSelection = "active" | "inactive" | "skipped"
    const imaged_wells = (wells: (WellInfo | null)[][]) => {
        let check = (w: any) => (w !== null ? "active" : "skipped") as WellSelection
        return wells.map((r) => r.map(check))
    }

    let active_wells = $state(imaged_wells(info.wells))
    const toggle_well = (r:number, c:number) => {
        let cur = active_wells[r][c]
        if (cur === "active") {
            active_wells[r][c] = "inactive"
        } else if (cur === "inactive") {
            active_wells[r][c] = "active"
        }
    }
    const set_active = (w: WellSelection) => {
        return w !== "skipped" ? "active" : w
    }

    const set_inactive = (w: WellSelection) => {
        return w !== "skipped" ? "inactive" : w
    }

    const toggle_row = (r:number) => {
        // if any off well, turn all on
        const turnOn = active_wells[r].some((w) => w === "inactive")
        const toggle = (w: WellSelection) => turnOn ? set_active(w) : set_inactive(w)
        active_wells[r] = active_wells[r].map(toggle)
    }
    const toggle_col = (c:number) => {
        const turnOn = active_wells
            .map((r) => r[c])
            .some((w) => w === "inactive")
        const toggle = (w: WellSelection) => turnOn ? set_active(w) : set_inactive(w)
        for (let r = 0; r <  active_wells.length; r++) {
            active_wells[r][c] = toggle(active_wells[r][c])
        }
    }
    const turn_plate_on = () => {
        active_wells = active_wells.map(r => r.map(set_active))
    }
    const turn_plate_off = () => {
        active_wells = active_wells.map(r => r.map(set_inactive))
    }
    const checkboard = () => {
        active_wells = active_wells.map((r, i) => {
            return r.map((w, j) => (i + j) % 2 ? set_inactive(w) : set_active(w) )
        })
    }
    const toggle_plate = () => {
        active_wells = active_wells.map((r) => {
            return r.map((w) => {
                return w === "active" ?
                    "inactive" :
                    w === "inactive" ?
                    "active" :
                    w
            })
        })
    }
    // Fields
    let selFields = $state({
        start: 1,
        end: info.fields,
        step: 1
    })

    // Planes
    let selPlanes = $state({
        start: 1,
        end: info.planes,
        step: 1
    })

    // create summary and send to rust...?
    const cnameToCid = Object.fromEntries(info.channels.map(ch => [ch.name, ch.id]))
    const apply_filter = async () => {
        const wells = active_wells.flatMap((r, i) => {
            return r.map((w, j) => [w, i, j])
            .filter(([w, ..._]) => w === 'active')
            .map(([_, i, j]) => [Number(i)+1, Number(j)+1])

        })
        const filter = {
            channels: active_channels.map(n => cnameToCid[n]),
            wells,
            fields: [...range(selFields.start, selFields.end+1, selFields.step)],
            planes: [...range(selPlanes.start, selPlanes.end+1, selPlanes.step)],
        }

        await invoke('set_filter', {filter})
        await goto('./output')
    }

</script>

<main>
    <h1>Select Images to Download</h1>
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
    <div class="plate">
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
        {#snippet well(r:number, c:number, selStatus: WellSelection)}
        <td class="{selStatus}" onclick={(_) => toggle_well(r,c)}></td>
        {/snippet}
    </WellPlate>
    </div>

    <button onclick={turn_plate_on}>All Wells</button>
    <button onclick={turn_plate_off}>No Wells</button>
    <button onclick={checkboard}>Checkboard Wells</button>
    <button onclick={toggle_plate}>Toggle Wells</button>

    <h3> Fields </h3>
    <p> {JSON.stringify(info.fields)} fields per well </p>
    <div class="range">          
        <label for="fStart">First Field: </label>
        <input id="fStart" type="number" bind:value={selFields.start} />
    

        <label for="fEnd">Last Field: </label>
        <input id="fEnd" type="number" bind:value={selFields.end} />

        <label for="fStep">Step by: </label>
        <input id="fStep" type="number" bind:value={selFields.step} />
    </div>

    <h3> Planes </h3>
    <p> {JSON.stringify(info.planes)} planes per field </p>
    <div class="range">
        <label for="pMin">Lowest: </label>
        <input id="pMin" type="number" bind:value={selPlanes.start} />

        <label for="pMax">Highest: </label>
        <input id="pMax" type="number" bind:value={selPlanes.end} />
        
        <label for="pStep">Step by: </label>
        <input id="pStep" type="number" bind:value={selPlanes.step} />
    </div>

    <div class="centered">
        <button class='next' onclick={apply_filter}>Select Output Options</button>
    </div>

    <hr />
    <a href="/">Select Another File?</a>
</main>

<style>
    div.plate {
        margin-bottom: 0.75rem;
    }
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
        background-color: slategray;
    }
    .skipped {
        background-color: #444;
    }
    label {
        cursor: pointer;
        display: inline-block;
        padding: 0.5rem 1rem;
    }
    div.range {
        display: flex;
    }
    .range input {
        max-width: 6rem;
    }
    button.next {
        margin-top: 2rem;
        margin-bottom: 0.5rem;
        width: 85%;
    }
    .centered {
        display: flex;
        justify-content: center;
        text-align: center;
    }
</style>


