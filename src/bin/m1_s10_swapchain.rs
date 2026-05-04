use learn_vulkan_renderer::{WindowConfig, run_swapchain_shell};

fn main() -> Result<(), learn_vulkan_renderer::SwapchainError> {
    run_swapchain_shell(WindowConfig {
        title: "Learn Vulkan Renderer - M1-S10".to_owned(),
        ..WindowConfig::default()
    })
}
