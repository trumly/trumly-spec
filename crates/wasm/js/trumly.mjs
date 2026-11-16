// Loads trumly.wasm and exposes `check(files)`. Works in browsers and Node 18+.
//
// A WASM module can only pass numbers, so text travels through its memory:
// copy the input in, call check_json, read the answer out, free both buffers.

export async function loadTrumly(wasmBytes) {
  const { instance } = await WebAssembly.instantiate(wasmBytes, {});
  const x = instance.exports;
  const enc = new TextEncoder();
  const dec = new TextDecoder();

  return {
    /** files: [[path, text], ...]  ->  the `check --json` document */
    check(files) {
      const input = enc.encode(JSON.stringify(files));
      const inPtr = x.alloc(input.length);
      // Re-read x.memory.buffer after every call into the module: it is replaced if memory grows.
      new Uint8Array(x.memory.buffer, inPtr, input.length).set(input);

      const packed = x.check_json(inPtr, input.length); // a BigInt: pointer in the high 32 bits, length in the low
      const outPtr = Number(packed >> 32n);
      const outLen = Number(packed & 0xffffffffn);
      const text = dec.decode(new Uint8Array(x.memory.buffer, outPtr, outLen));

      x.dealloc(outPtr, outLen);
      x.dealloc(inPtr, input.length);
      const result = JSON.parse(text);
      if (result.error) throw new Error(result.error);
      return result;
    },

    /** One of the agent tools (the MCP server's read tools): files, a tool name and its arguments -> the tool's result. Throws the tool's error message. */
    tool(files, tool, args = {}) {
      const input = enc.encode(JSON.stringify({ files, tool, args }));
      const inPtr = x.alloc(input.length);
      new Uint8Array(x.memory.buffer, inPtr, input.length).set(input);
      const packed = x.tool_json(inPtr, input.length);
      const outPtr = Number(packed >> 32n);
      const outLen = Number(packed & 0xffffffffn);
      const text = dec.decode(new Uint8Array(x.memory.buffer, outPtr, outLen));
      x.dealloc(outPtr, outLen);
      x.dealloc(inPtr, input.length);
      const out = JSON.parse(text);
      if (out.error !== undefined) throw new Error(out.error);
      return out.result;
    },
  };
}
