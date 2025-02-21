<script lang="ts">
    import { type DLEvent, type DownloadInfo } from "$lib/ffi_types"
    import { range } from "$lib/range";
    import { Channel, invoke } from "@tauri-apps/api/core";
    import WellPlate from "../WellPlate.svelte";
    import { goto } from "$app/navigation";

    let { data }: {data: {info: DownloadInfo}} = $props();
    let info = data.info;
    let max_planes = (() => {
        let f = info.filter
        // timepoints?
        return f.channels.length * f.fields.length * f.planes.length
    })() 

    interface WellStatus {
        progress: "skipped" | "waiting" | "processing" | "finished",
        planes: number,
    }

    function create_status() {
        let wells: (WellStatus)[][] = [...range(info.rows)]
            .map(_ => [...range(info.cols)].map(_ => ({
                progress: 'skipped',
                planes: 0,
            })))

        for (const [r, c] of data.info.filter.wells) {
            wells[r-1][c-1].progress = "waiting"
        }


        return wells
    }
    console.log(info)

    let wellStatus = $state(create_status())
    let dlStatus: "W" | "R" | "C" = $state("W")

    // start the image downloads
    const onEvent = new Channel<DLEvent>()
    const handle_dl_event = (msg: DLEvent, status: WellStatus[][]) => {
        switch (msg.event) {
            case "started": {
                dlStatus = 'R'
                break;
            }
            case "plane": {
                let {r, c} = msg.data
                let w = wellStatus[r - 1][c - 1]

                w.planes += 1
                w.progress = w.planes >= max_planes ? 'finished' : 'processing'
                break;
            }
            case "finished": {
                dlStatus = 'C'
                break;
            }
        }
    }
    onEvent.onmessage = (msg) => {
        handle_dl_event(msg, wellStatus)
    }

    async function download_plz() {
        return invoke<null>('start_download', {onEvent: onEvent})
            .then(_ => console.log('download complete!'))
            .catch(err => {throw err})
    }

    function display_well_status(well: WellStatus) {
        let fmt = new Intl.NumberFormat(undefined, {
            style: 'percent',
            maximumFractionDigits: 0
        })
        return well.progress === "processing" ?
            fmt.format(well.planes / max_planes)
            :
            ""
    }

    function display_dl_status() {
        switch(dlStatus) {
            case "W": return "Waiting..."
            case "R": return "Downloading"
            case "C": return "Dowload Complete!"
        }
    }

    async function clear_and_restart() {
        await invoke('reset_state')
        await goto('/')
    }


</script>

<main>
    <h1>{info.name}</h1>
    <p>Downloading to:</p> 
    <p>{info.output.dir}</p>

    {#if dlStatus === "W"}
        <button onclick={download_plz}>Start Download</button>
    {/if}

    <h2>
        {display_dl_status()}
    </h2>

    {#if dlStatus === "C"}
        <button onclick={clear_and_restart}>Select Another Plate</button>
    {/if}

    <WellPlate plate={wellStatus}>
        {#snippet rowHdr(r: number)}
            <th scope="row">{String.fromCharCode(65+r)}</th>
        {/snippet}
        {#snippet colHdr(c: number)}
            <th scope="col">{c + 1}</th>
        {/snippet}
        {#snippet well(r: number, c: number, s: WellStatus)}
            <td class={s.progress}>{display_well_status(s)}</td>
        {/snippet}
    </WellPlate>
</main>

<footer>
    <hr />
    <a href="/output">← Change output info</a>
</footer>


<style>
    td {
        min-width: 2.5rem;
        text-align: center;
        vertical-align: center;
    }
    td.skipped {
        background-color: darkgray;
    }
    td.waiting {
        background-color: lightblue;
    }
    td.processing {
        background-color: dodgerblue;
    }
    td.finished {
        background-color: olivedrab;
    }
</style>
