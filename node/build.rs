#![warn(
	clippy::disallowed_method,
	clippy::indexing_slicing,
	clippy::todo,
	clippy::unwrap_used,
	clippy::panic
)]
use hydra_dx_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
	generate_cargo_keys("basilisk-runtime").expect("Failed to generate version metadata");
	rerun_if_git_head_changed();
}
