<script lang="ts">
    import type { DownloadInfo } from "$lib/ffi_types"

    let { data }: {data: {info: DownloadInfo}} = $props();
    let info = data.info;


</script>