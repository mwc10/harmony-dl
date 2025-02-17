import type { DownloadInfo } from '$lib/ffi_types';
import type { PageLoad } from './$types'

import { invoke } from "@tauri-apps/api/core";

export const load: PageLoad = async (_e) => {
    return {
        info: await invoke<DownloadInfo>('get_dl_info')
    }
}

