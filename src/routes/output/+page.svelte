<script lang="ts">
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
</script>