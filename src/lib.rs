pub mod rpc;

// Re-export for convenience (optional but nice)
// re-exports the function (so users can do soltop::get_rpc_url instead of soltop::rpc::get_rpc_url)
pub use rpc::get_rpc_url;

pub fn hello_from_lib() -> String {
    "Hello from soltop library!".to_string()
}