//! trumly-core for the browser. No wasm-bindgen: three plain functions, so the only tool needed is
//! `cargo build --target wasm32-unknown-unknown`.
//!
//! A WASM module can only exchange numbers with JavaScript, so text goes through the module's
//! memory: JS asks us to `alloc` a buffer, copies bytes in, calls `check`, reads the answer bytes
//! back out of memory, then calls `dealloc` on both buffers.
//!
//!   check_json   input  (JSON): [["path.md", "file text"], ...]
//!                output (JSON): the `check --json` document, or {"error": "..."}
//!   tool_json    input  (JSON): {"files": [["path.md", "file text"], ...], "tool": "search", "args": {...}}
//!                output (JSON): {"result": ...} or {"error": "..."}. The agent tools of the MCP server, so the hosted
//!                server answers exactly as the local one does.

use trumly_core::report::check;
use trumly_core::tools;

/// Reserve `len` bytes inside the module and return where they are.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf); // we hand ownership to the caller until they call dealloc
    ptr
}

/// Give a buffer back. `len` must be the length passed to `alloc` (or returned by `check`).
///
/// # Safety
/// `ptr` and `len` must come from `alloc` or `check` and be freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
}

/// Run `check` over the JSON input at (ptr, len). Returns the output location packed into one
/// u64: high 32 bits = pointer, low 32 bits = length. (wasm32 pointers are 32 bits.)
///
/// # Safety
/// `ptr..ptr+len` must be readable memory, as filled in by the caller after `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn check_json(ptr: *const u8, len: usize) -> u64 {
    let input = unsafe { std::slice::from_raw_parts(ptr, len) };
    let out = run(input);
    let mut bytes = out.into_bytes().into_boxed_slice();
    let (p, l) = (bytes.as_mut_ptr() as usize as u64, bytes.len() as u64);
    std::mem::forget(bytes);
    (p << 32) | l
}

/// Run one agent tool (see `trumly_core::tools`) over the JSON input at (ptr, len). Packed the same way as `check_json`.
///
/// # Safety
/// `ptr..ptr+len` must be readable memory, as filled in by the caller after `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tool_json(ptr: *const u8, len: usize) -> u64 {
    let input = unsafe { std::slice::from_raw_parts(ptr, len) };
    let out = run_tool(input);
    let mut bytes = out.into_bytes().into_boxed_slice();
    let (p, l) = (bytes.as_mut_ptr() as usize as u64, bytes.len() as u64);
    std::mem::forget(bytes);
    (p << 32) | l
}

/// The safe part of `tool_json`.
pub fn run_tool(input: &[u8]) -> String {
    let v: serde_json::Value = match serde_json::from_slice(input) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({ "error": e.to_string() }).to_string(),
    };
    let files: Result<Vec<(String, String)>, String> =
        serde_json::from_value(v["files"].clone()).map_err(|e| e.to_string());
    let Ok(files) = files else {
        return serde_json::json!({ "error": "files must be a list of [path, text] pairs" }).to_string();
    };
    let Some(tool) = v["tool"].as_str() else {
        return serde_json::json!({ "error": "a tool name is needed" }).to_string();
    };
    let call = (files, tool.to_string(), v["args"].clone());
    let report = check(call.0);
    match tools::call(&report, &call.1, &call.2) {
        Some(Ok(v)) => serde_json::json!({ "result": v }).to_string(),
        Some(Err(e)) => serde_json::json!({ "error": e }).to_string(),
        None => serde_json::json!({ "error": format!("unknown tool '{}'", call.1) }).to_string(),
    }
}

/// The safe part, separated so it can be tested without any pointers.
pub fn run(input: &[u8]) -> String {
    let files: Result<Vec<(String, String)>, String> =
        serde_json::from_slice::<Vec<(String, String)>>(input).map_err(|e| e.to_string());
    match files {
        Ok(files) => check(files).to_json().to_string(),
        Err(e) => serde_json::json!({ "error": e }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_a_model_given_as_json() {
        let out = run(br#"[["a.md", "---\nid: a\ntype: t\nparent: nope\n---\n"]]"#);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["structure"]["errors"][0]["kind"], "missing_parent");
    }

    #[test]
    fn a_tool_runs_over_files_given_as_json() {
        let files = r#"[["a.md", "---\nid: a\ntype: goal\ntitle: Ship it\n---\nhello world\n"]]"#;
        let out =
            run_tool(format!(r#"{{"files": {files}, "tool": "search", "args": {{"query": "WORLD"}}}}"#).as_bytes());
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["result"]["results"][0]["id"], "a");
        let none: serde_json::Value =
            serde_json::from_str(&run_tool(format!(r#"{{"files": {files}, "tool": "nope"}}"#).as_bytes())).unwrap();
        assert!(none["error"].as_str().unwrap().contains("unknown tool"));
    }

    #[test]
    fn bad_input_is_an_error_object_not_a_crash() {
        let v: serde_json::Value = serde_json::from_str(&run(b"nonsense")).unwrap();
        assert!(v["error"].is_string());
    }

    // The pointer functions pack a 32-bit address into a u64, which only makes sense inside
    // wasm32. So this test runs there (via a wasm test runner) and is skipped on a normal machine.
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn the_pointer_api_round_trips() {
        let input = br#"[["a.md", "---\nid: a\ntype: t\n---\n"]]"#;
        let buf = alloc(input.len());
        unsafe {
            std::ptr::copy_nonoverlapping(input.as_ptr(), buf, input.len());
            let packed = check_json(buf, input.len());
            let (p, l) = ((packed >> 32) as usize, (packed & 0xffff_ffff) as usize);
            let out = std::slice::from_raw_parts(p as *const u8, l);
            assert!(std::str::from_utf8(out).unwrap().contains("\"nodes\""));
            dealloc(p as *mut u8, l);
            dealloc(buf, input.len());
        }
    }
}
