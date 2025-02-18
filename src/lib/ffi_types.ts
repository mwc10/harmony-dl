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
    rows: number,
    cols: number,
    output: OutputInfo,
    filter: ImageFilter,
}

export type DLEvent = 
| {
    event: 'started';
    data: {}
  }
| {
    event: 'plane';
    data: {
        r: number,
        c: number,
        f: number,
        p: number,
    };
  }
| {
    event: 'finished';
    data: {}
 }; 

