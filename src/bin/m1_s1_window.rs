use learn_vulkan_renderer::{WindowConfig, run_window_shell};

fn main() -> Result<(), winit::error::EventLoopError> {
    run_window_shell(WindowConfig::default())
}
