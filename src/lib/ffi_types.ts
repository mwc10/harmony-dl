export interface XmlInfo {
    name: string,
    rows: number,
    cols: number,
    fields: number,
    planes: number,
    timepoints: number,
    wells: (WellInfo | null)[][],
    channels: Channel[],
}

export interface WellInfo {
    row: number,
    col: number,
    fields: number[],
    planes: number[],
    timepoints: number[]
}

export interface Channel {
    id: number,
    name: string,
    res: [number, number],
    mag: number
}

export interface ImageFilter {
    channels: number[],
    wells: [number, number][], // [r, c]
    fields: number[],
    planes: number[],
}

export interface OutputInfo {
    dir: string,
    action: string,
    format: string,
}

export interface DownloadInfo {
    name: string,
    output: OutputInfo,
    rows: number,
    cols: number,
    fields: number,
    planes: number,
    timepoints: number,
    wells: [number, number][],
}

