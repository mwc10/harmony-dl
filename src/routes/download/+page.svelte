<script lang="ts">
    import { type DLEvent, type DownloadInfo } from "$lib/ffi_types"
    import { range } from "$lib/range";
    import { Channel, invoke } from "@tauri-apps/api/core";
    import WellPlate from "../WellPlate.svelte";

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
    let dlStatus = $state("Waiting...")

    // start the image downloads
    const onEvent = new Channel<DLEvent>()
    const handle_dl_event = (msg: DLEvent, status: WellStatus[][]) => {
        switch (msg.event) {
            case "started": {
                dlStatus = 'Download Started!'
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
                dlStatus = 'Download Finished!'
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


</script>

<main>
    <h1>{info.name}</h1>
    <p>Downloading to: {info.output.dir}</p>

    <button onclick={download_plz}>Start Download</button>

    <h2>{dlStatus}</h2>

    <p>{JSON.stringify(wellStatus)}</p>

    <WellPlate plate={wellStatus}>
        {#snippet rowHdr(r: number)}
            <th scope="row">{r + 1}</th>
        {/snippet}
        {#snippet colHdr(c: number)}
            <th scope="col">{c + 1}</th>
        {/snippet}
        {#snippet well(r: number, c: number, s: WellStatus)}
            <td class={s.progress}>{display_well_status(s)}</td>
        {/snippet}
    </WellPlate>

</main>

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
