<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";

    let outdir: string | null = $state(null)
    let err = $state(null)

    async function select_output_dir() {
        const dir = await open({
            multiple: false,
            directory: true,
        })

        if (dir) {
            outdir = dir
        }
    }
    // processing
    const pipelines = ['Max Projection', 'Individual Planes']
    let action = $state(pipelines[0])
    // formats
    const formats = ['TIFF', 'OME-Zarr']
    let format = $state(formats[0])

    // saving and starting...
    async function start_download() {
        await invoke('set_output', {
            info: {
                dir: outdir,
                action,
                format
            }
        })

        await goto('./download')
    }
</script>

<main>
<h1>Output Settings</h1>

{#if outdir === null}
    <button onclick={select_output_dir}>Select Output Directory</button>
{:else}
    <div>Output Directory:</div>
    <div>{outdir}</div>

    <button onclick={start_download}>Download Images</button>
    <button onclick={() => outdir = null}>Select Another Directory</button>
{/if}

<h2> Output Format </h2>
{#each formats as fmt}
<label>
    <input 
        type="radio"
        name="formats"
        value={fmt}
        bind:group={format}
    />
    <span>{fmt}</span>
</label>
{/each}

<h2> Image Processing Pipeline </h2>
{#each pipelines as pipe}
<label>
    <input 
        type="radio"
        name="pipeline"
        value={pipe}
        bind:group={action}
    />
    <span>{pipe}</span>
</label>
{/each}




</main>

<style>
input, label {
    cursor: pointer;
}

label {
    padding: 0rem;
    padding-left: 0.25rem;
    padding-right: 1.25rem;
}

label>span:hover {
    text-decoration: underline;
}
</style>
