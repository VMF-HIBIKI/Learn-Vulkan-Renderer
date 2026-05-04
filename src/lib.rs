//! Rust Vulkan renderer learning crate.
//!
//! 这个 crate 先作为学习型渲染器的代码实验场存在。具体 Vulkan 绑定、
//! 资源封装、渲染图和光追模块会随着 `Docs/Vulkan-Learning` 的计划逐步落地。

pub mod commands;
pub mod device;
pub mod instance;
pub mod surface;
pub mod swapchain;
pub mod window;

pub use commands::{
    ClearCommandBundle, CommandError, create_clear_command_bundle, run_clear_command_shell,
};
pub use device::{
    DeviceError, DeviceExtensionSupport, LogicalDevice, PhysicalDeviceFeatureMatrix,
    PhysicalDeviceInfo, QueueFamilyIndices, QueueFamilyReport, QueueFamilySupport,
    RayTracingFeatureSupport, SelectedPhysicalDevice, SwapchainSupportSummary,
    create_logical_device, enumerate_physical_devices, query_physical_device_feature_matrix,
    query_queue_family_support, query_swapchain_support_summary, run_feature_matrix_shell,
    run_logical_device_shell, run_physical_device_shell, run_queue_support_shell,
    select_physical_device,
};
pub use instance::{
    VulkanInstance, VulkanInstanceConfig, VulkanInstanceError, run_instance_shell,
    run_validation_shell,
};
pub use surface::{SurfaceBootstrap, SurfaceError, run_surface_shell};
pub use swapchain::{
    SwapchainBundle, SwapchainConfig, SwapchainError, SwapchainSupportDetails,
    choose_swapchain_config, create_swapchain_bundle, query_swapchain_support_details,
    run_resizable_swapchain_shell, run_swapchain_config_shell, run_swapchain_shell,
};
pub use window::{WindowConfig, run_window_shell};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        assert_eq!(env!("CARGO_PKG_NAME"), "learn-vulkan-renderer");
    }
}
