use learn_vulkan_renderer::{WindowConfig, run_clear_command_shell};

fn main() -> Result<(), learn_vulkan_renderer::CommandError> {
    run_clear_command_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S12".to_owned(),
        ..WindowConfig::default()
    })
}
