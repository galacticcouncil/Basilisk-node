#![warn(
	clippy::disallowed_method,
	clippy::indexing_slicing,
	clippy::todo,
	clippy::unwrap_used,
	clippy::panic
)]
use substrate_wasm_builder::WasmBuilder;

fn main() {
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build()
}
