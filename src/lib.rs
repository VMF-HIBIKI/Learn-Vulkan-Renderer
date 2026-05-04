//! Rust Vulkan renderer learning crate.
//!
//! 这个 crate 先作为学习型渲染器的代码实验场存在。具体 Vulkan 绑定、
//! 资源封装、渲染图和光追模块会随着 `Docs/Vulkan-Learning` 的计划逐步落地。

pub mod device;
pub mod instance;
pub mod surface;
pub mod window;

pub use device::{
    DeviceError, PhysicalDeviceInfo, enumerate_physical_devices, run_physical_device_shell,
};
pub use instance::{
    VulkanInstance, VulkanInstanceConfig, VulkanInstanceError, run_instance_shell,
    run_validation_shell,
};
pub use surface::{SurfaceBootstrap, SurfaceError, run_surface_shell};
pub use window::{WindowConfig, run_window_shell};

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        assert_eq!(env!("CARGO_PKG_NAME"), "learn-vulkan-renderer");
    }
}
