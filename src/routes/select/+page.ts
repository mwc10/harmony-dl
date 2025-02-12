import type { XmlInfo } from '$lib/ffi_types';
import type { PageLoad } from './$types'

import { invoke } from "@tauri-apps/api/core";

export const load: PageLoad = async (_e) => {
    return {
        info: await invoke<XmlInfo>('get_info')
    }
}

