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

  function range(size: number, startAt = 0) {
    return [...Array(size).keys()].map(i => i + startAt);
  }
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

:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

td:hover {
  background-color: red;
  cursor: pointer;
}
ul {
  padding: 0rem;
  marker: none;
}
li {
  display: inline;
  padding: 0rem 2rem;
  margin: 1rem 0.5rem;
}
li:hover {
  background-color: purple;
  color: white;
  cursor: pointer;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
