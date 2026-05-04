use learn_vulkan_renderer::{WindowConfig, run_logical_device_shell};

fn main() -> Result<(), learn_vulkan_renderer::DeviceError> {
    run_logical_device_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S8".to_owned(),
        ..WindowConfig::default()
    })
}
