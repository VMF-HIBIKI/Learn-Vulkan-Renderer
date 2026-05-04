use learn_vulkan_renderer::{WindowConfig, run_queue_support_shell};

fn main() -> Result<(), learn_vulkan_renderer::DeviceError> {
    run_queue_support_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S6".to_owned(),
        ..WindowConfig::default()
    })
}
