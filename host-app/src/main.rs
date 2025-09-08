use wasmtime::*;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi::p2 as wasi_p2;
use std::fs;

// Include the generated bindings
wasmtime::component::bindgen!({
    world: "foo-world",
    path: "wit"
});

// Set up WASI Preview 2 context (env + stdio) and state for the Store
struct HostState {
    table: ResourceTable,
    wasi: WasiCtx,
}

impl WasiView for HostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView { ctx: &mut self.wasi, table: &mut self.table }
    }
}

fn main() -> anyhow::Result<()> {
    // Enable comprehensive wasmtime debug logging
    std::env::set_var("WASMTIME_BACKTRACE_DETAILS", "1");
    std::env::set_var("WASMTIME_LOG", "debug");
    std::env::set_var("RUST_LOG", "debug");
    
    // Initialize logging
    env_logger::init();
    
    println!("🔧 Starting Wasmtime host app with debug enabled...");

    // Load the component
    let component_bytes = fs::read("../wasm-component/target/wasm32-wasip2/debug/wasm_component.wasm")?;
    println!("📦 Loaded component: {} bytes", component_bytes.len());
    
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.debug_info(true);           // Enable debug info
    config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
    config.cranelift_debug_verifier(true);  // Enable Cranelift verification
    config.cranelift_opt_level(OptLevel::None);
    
    let engine = Engine::new(&config)?;
    println!("🚀 Engine created with debug configuration");
    
    let component = Component::new(&engine, &component_bytes)?;
    println!("🧩 Component instantiated successfully");
    
    let table = ResourceTable::new();
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env() // provide environment variables
        .build();

    let mut store = Store::new(&engine, HostState { table, wasi });

    let mut linker = Linker::new(&engine);
    println!("🔗 Linker created");
    // Provide wasi:cli (and its dependencies like environment) to the component
    wasi_p2::add_to_linker_sync(&mut linker)?;
    
    let bindings = FooWorld::instantiate(&mut store, &component, &linker)?;
    println!("⚡ Component bindings established");
    
    let input = 41;
    println!("📞 Calling foo({})...", input);
    
    let result = bindings.component_foo_math().call_foo(&mut store, input)?;
    
    println!("✅ foo({}) = {}", input, result);
    
    // Test the new bar() function
    let keys = vec!["username".to_string(), "config".to_string(), "session".to_string()];
    println!("📞 Calling bar({:?})...", keys);
    
    let key_values = bindings.component_foo_math().call_bar(&mut store, &keys)?;
    
    println!("✅ bar() returned {} key-value pairs:", key_values.len());
    for kv in key_values {
        println!("  📋 {} -> {}", kv.key, kv.value);
    }
    
    Ok(())
}