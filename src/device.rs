use std::{
    error::Error,
    ffi::CStr,
    fmt::{Debug, Display, Formatter},
};

use ash::vk;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasDisplayHandle,
    window::{Window, WindowId},
};

use crate::{VulkanInstance, VulkanInstanceError, WindowConfig};

#[derive(Debug)]
pub enum DeviceError {
    Instance(VulkanInstanceError),
    Vulkan(vk::Result),
    NoPhysicalDevices,
}

impl Display for DeviceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instance(error) => write!(formatter, "Vulkan instance error: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
            Self::NoPhysicalDevices => write!(formatter, "no Vulkan physical devices found"),
        }
    }
}

impl Error for DeviceError {}

impl From<VulkanInstanceError> for DeviceError {
    fn from(error: VulkanInstanceError) -> Self {
        Self::Instance(error)
    }
}

impl From<vk::Result> for DeviceError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalDeviceInfo {
    pub handle: vk::PhysicalDevice,
    pub name: String,
    pub device_type: vk::PhysicalDeviceType,
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
}

impl PhysicalDeviceInfo {
    fn from_properties(
        handle: vk::PhysicalDevice,
        properties: vk::PhysicalDeviceProperties,
    ) -> Self {
        // SAFETY: Vulkan guarantees `device_name` is a nul-terminated fixed-size C string.
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();

        Self {
            handle,
            name,
            device_type: properties.device_type,
            api_version: properties.api_version,
            driver_version: properties.driver_version,
            vendor_id: properties.vendor_id,
            device_id: properties.device_id,
        }
    }

    pub fn api_version_string(&self) -> String {
        format!(
            "{}.{}.{}",
            vk::api_version_major(self.api_version),
            vk::api_version_minor(self.api_version),
            vk::api_version_patch(self.api_version)
        )
    }
}

impl Display for PhysicalDeviceInfo {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} ({:?}, Vulkan {}, vendor {:#06x}, device {:#06x}, driver {})",
            self.name,
            self.device_type,
            self.api_version_string(),
            self.vendor_id,
            self.device_id,
            self.driver_version
        )
    }
}

pub fn enumerate_physical_devices(
    vulkan_instance: &VulkanInstance,
) -> Result<Vec<PhysicalDeviceInfo>, DeviceError> {
    // SAFETY: `vulkan_instance` owns a live `VkInstance`; enumeration writes into ash-managed Vec.
    let physical_devices = unsafe { vulkan_instance.handle().enumerate_physical_devices()? };

    if physical_devices.is_empty() {
        return Err(DeviceError::NoPhysicalDevices);
    }

    Ok(physical_devices
        .into_iter()
        .map(|physical_device| {
            // SAFETY: `physical_device` was returned by this live instance.
            let properties = unsafe {
                vulkan_instance
                    .handle()
                    .get_physical_device_properties(physical_device)
            };

            PhysicalDeviceInfo::from_properties(physical_device, properties)
        })
        .collect())
}

pub fn run_physical_device_shell(config: WindowConfig) -> Result<(), DeviceError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = PhysicalDeviceShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)?;
    app.result
}

#[derive(Debug)]
struct PhysicalDeviceShell {
    config: WindowConfig,
    vulkan_instance: Option<VulkanInstance>,
    window: Option<Window>,
    result: Result<(), DeviceError>,
}

impl PhysicalDeviceShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            vulkan_instance: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_window_instance_and_list_devices(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), DeviceError> {
        if self.vulkan_instance.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(self.config.attributes())
                    .map_err(VulkanInstanceError::from)?,
            );
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before physical device enumeration");
        let display_handle = window
            .display_handle()
            .map_err(VulkanInstanceError::from)?
            .as_raw();
        let vulkan_instance =
            VulkanInstance::new_for_display(display_handle, "learn-vulkan-renderer-m1-s5")?;
        let physical_devices = enumerate_physical_devices(&vulkan_instance)?;

        println!(
            "M1-S5 found {} Vulkan physical device(s)",
            physical_devices.len()
        );
        for (index, physical_device) in physical_devices.iter().enumerate() {
            println!("  [{index}] {physical_device}");
        }

        self.vulkan_instance = Some(vulkan_instance);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: DeviceError) {
        eprintln!("M1-S5 physical device enumeration failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for PhysicalDeviceShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_window_instance_and_list_devices(event_loop) {
            self.record_error_and_exit(event_loop, error);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = &self.window else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        if matches!(event, WindowEvent::CloseRequested) {
            println!("M1-S5 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.vulkan_instance = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.vulkan_instance = None;
    }
}
