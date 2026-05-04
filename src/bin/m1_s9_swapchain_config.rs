use learn_vulkan_renderer::{WindowConfig, run_swapchain_config_shell};

fn main() -> Result<(), learn_vulkan_renderer::SwapchainError> {
    run_swapchain_config_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S9".to_owned(),
        ..WindowConfig::default()
    })
}
