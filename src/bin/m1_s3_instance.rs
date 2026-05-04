use learn_vulkan_renderer::{WindowConfig, run_instance_shell};

fn main() -> Result<(), learn_vulkan_renderer::VulkanInstanceError> {
    run_instance_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S3".to_owned(),
        ..WindowConfig::default()
    })
}
