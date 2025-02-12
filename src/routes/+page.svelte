<script lang='ts'>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { goto } from '$app/navigation'

  let file_path = $state("")
  let err = $state(null)

  async function open_xml() {
    const file = await open({
      multiple: false,
      directory: false,
    })

    err = null

    if (file) {
      file_path = file
      try {
        await invoke<[]>('parse_xml', {path: file_path});

        await goto('./select')

      } catch (e: any) {
        err = e
      }
    }
  }

  // async function select_output_and_export() {
  //   const output = await open({
  //     multiple: false,
  //     directory: true,
  //   })

  //   err = null

  //   if (output) {
  //     outdir = output

  //     invoke<number>('test_download', {outdir: output})
  //       .then((n) => images_downloaded = n)
  //       .catch((e) => err = e)
  //   }
  // }
</script>

<main class="container">
  <h1>Harmony Image Downloader</h1>
  <button onclick={open_xml}>Select Export XML</button>

  {#if err !== null}
    <h2> ERROR ! </h2>
    <p style="white-space: pre-wrap"> {err} </p>
  {:else if file_path !== ""}
    <p> XML File: {file_path} </p>
    <p>...Parsing XML...</p>
  {/if}
</main>

<style>
  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }
</style>
