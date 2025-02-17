<script lang="ts">
    import { type DLEvent, type DownloadInfo } from "$lib/ffi_types"
    import { range } from "$lib/range";
    import { Channel, invoke } from "@tauri-apps/api/core";
    import WellPlate from "../WellPlate.svelte";

    let { data }: {data: {info: DownloadInfo}} = $props();
    let info = data.info;

    interface WellStatus {
        progress: "skipped" | "waiting" | "processing" | "finished",
        timepoints: number,
        fields: number,
        planes: number,
    }

    function create_status() {
        let wells: (WellStatus)[][] = [...range(info.rows)]
            .map(_ => [...range(info.cols)].map(_ => ({
                progress: 'skipped',
                timepoints: 0,
                fields: 0,
                planes: 0,
            })))

        for (const [r, c] of data.info.filter.wells) {
            wells[r-1][c-1].progress = "waiting"
        }


        return wells
    }
    console.log(info)

    let status = $state(create_status())

    // start the image downloads
    const onEvent = new Channel<DLEvent>()
    const handle_dl_event = (msg: DLEvent, status: WellStatus[][]) => {
        let {r, c} = msg.data
        r = r - 1
        c = c - 1
        switch (msg.event) {
            case "started": {
                status[r][c].progress = "processing"
                break;
            }
            case "field": {
                if (status[r][c].progress !== "skipped") {
                    status[r][c].fields = status[r][c].fields + 1 
                }
                break;
            }
            case "finished": {
                status[r][c].progress = "finished"
                break;
            }
        }
    }
    onEvent.onmessage = (msg) => {
        console.log(`got message <${msg.event}>`)
        handle_dl_event(msg, status)
    }

    async function download_plz() {
        return invoke<null>('start_download', {onEvent: onEvent})
            .then(_ => console.log('download complete!'))
            .catch(err => {throw err})
    }


</script>

<main>
    <h1>{info.name}</h1>
    <p>Downloading to: {info.output.dir}</p>

    <button onclick={download_plz}>Start Download</button>

    <p>{JSON.stringify(status)}</p>

    <WellPlate plate={status}>
        {#snippet rowHdr(r: number)}
            <th scope="row">{r + 1}</th>
        {/snippet}
        {#snippet colHdr(c: number)}
            <th scope="col">{c + 1}</th>
        {/snippet}
        {#snippet well(r: number, c: number, s: WellStatus)}
            <td class={s.progress}>{s.progress === "skipped" ? "" : s.progress}</td>
        {/snippet}
    </WellPlate>

</main>

<style>
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
        background-color: green;
    }
</style>
