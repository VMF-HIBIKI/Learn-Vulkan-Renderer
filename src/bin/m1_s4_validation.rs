use learn_vulkan_renderer::{WindowConfig, run_validation_shell};

fn main() -> Result<(), learn_vulkan_renderer::VulkanInstanceError> {
    run_validation_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S4".to_owned(),
        ..WindowConfig::default()
    })
}
