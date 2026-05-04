use learn_vulkan_renderer::{WindowConfig, run_present_loop_shell};

fn main() -> Result<(), learn_vulkan_renderer::CommandError> {
    run_present_loop_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S13".to_owned(),
        ..WindowConfig::default()
    })
}
