export function* range(a: number, end:number|null = null, step:number|null = null) {
    const start = end === null ? 0 : a
    end = end === null ? a : end
    step = step === null ? 1 : step

    for (let i = start; i < end; i += step) {
        yield i
    }
}
