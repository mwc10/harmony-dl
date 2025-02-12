<script lang='ts'>
    function range(size: number, startAt = 0) {
        return [...Array(size).keys()].map(i => i + startAt);
    }

    const { plate, well, rowHdr, colHdr }: {
        plate: (any|null)[][],
        well: any,
        rowHdr: any,
        colHdr: any,
    } = $props()
    const rows = plate.length
    const cols = rows > 0 ? plate[0].length : 0
</script>

<table>
    <tbody>
        <tr>
            <th scope="col"></th>
            {#each range(cols) as colnum}
            {@render colHdr(colnum)}
            {/each}
        </tr>
        {#each plate as _row, r}
        <tr>
            {@render rowHdr(r)}
            {#each plate[r] as active, c}
            {@render well(r, c, active)}
            {/each}
        </tr>
        {/each}
    </tbody>
</table>
