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

use crate::{SurfaceBootstrap, SurfaceError, VulkanInstance, VulkanInstanceError, WindowConfig};

#[derive(Debug)]
pub enum DeviceError {
    Instance(VulkanInstanceError),
    Surface(SurfaceError),
    Vulkan(vk::Result),
    NoPhysicalDevices,
    NoSuitableQueueFamilies,
}

impl Display for DeviceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instance(error) => write!(formatter, "Vulkan instance error: {error}"),
            Self::Surface(error) => write!(formatter, "Vulkan surface error: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
            Self::NoPhysicalDevices => write!(formatter, "no Vulkan physical devices found"),
            Self::NoSuitableQueueFamilies => {
                write!(
                    formatter,
                    "no device has both graphics and present queue support"
                )
            }
        }
    }
}

impl Error for DeviceError {}

impl From<VulkanInstanceError> for DeviceError {
    fn from(error: VulkanInstanceError) -> Self {
        Self::Instance(error)
    }
}

impl From<SurfaceError> for DeviceError {
    fn from(error: SurfaceError) -> Self {
        Self::Surface(error)
    }
}

impl From<vk::Result> for DeviceError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct QueueFamilyReport {
    pub family_index: u32,
    pub queue_count: u32,
    pub queue_flags: vk::QueueFlags,
    pub supports_present: bool,
}

#[derive(Debug, Clone)]
pub struct QueueFamilySupport {
    pub reports: Vec<QueueFamilyReport>,
    pub indices: QueueFamilyIndices,
}

#[derive(Debug, Clone)]
pub struct SwapchainSupportSummary {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: vk::Extent2D,
    pub format_count: usize,
    pub present_mode_count: usize,
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

pub fn query_queue_family_support(
    vulkan_instance: &VulkanInstance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<QueueFamilySupport, DeviceError> {
    // SAFETY: `physical_device` was returned by this instance; the call only reads properties.
    let queue_families = unsafe {
        vulkan_instance
            .handle()
            .get_physical_device_queue_family_properties(physical_device)
    };
    let mut indices = QueueFamilyIndices::default();
    let mut reports = Vec::with_capacity(queue_families.len());

    for (family_index, queue_family) in queue_families.iter().enumerate() {
        let family_index = family_index as u32;

        // SAFETY: surface belongs to the same live instance, and physical_device/family index came
        // from Vulkan queries on that instance.
        let supports_present = unsafe {
            surface_loader.get_physical_device_surface_support(
                physical_device,
                family_index,
                surface,
            )?
        };

        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && indices.graphics_family.is_none()
        {
            indices.graphics_family = Some(family_index);
        }

        if queue_family.queue_count > 0 && supports_present && indices.present_family.is_none() {
            indices.present_family = Some(family_index);
        }

        reports.push(QueueFamilyReport {
            family_index,
            queue_count: queue_family.queue_count,
            queue_flags: queue_family.queue_flags,
            supports_present,
        });
    }

    Ok(QueueFamilySupport { reports, indices })
}

pub fn query_swapchain_support_summary(
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<SwapchainSupportSummary, DeviceError> {
    // SAFETY: surface belongs to the live instance that enumerated `physical_device`.
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    // SAFETY: this only queries formats supported by the device/surface pair.
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface)? };
    // SAFETY: this only queries present modes supported by the device/surface pair.
    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
    };

    Ok(SwapchainSupportSummary {
        min_image_count: capabilities.min_image_count,
        max_image_count: capabilities.max_image_count,
        current_extent: capabilities.current_extent,
        format_count: formats.len(),
        present_mode_count: present_modes.len(),
    })
}

pub fn run_physical_device_shell(config: WindowConfig) -> Result<(), DeviceError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = PhysicalDeviceShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)?;
    app.result
}

pub fn run_queue_support_shell(config: WindowConfig) -> Result<(), DeviceError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = QueueSupportShell::new(config);

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

#[derive(Debug)]
struct QueueSupportShell {
    config: WindowConfig,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), DeviceError>,
}

impl QueueSupportShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_surface_and_report_support(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), DeviceError> {
        if self.surface.is_some() {
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
            .expect("window was created before queue support query");
        let surface = SurfaceBootstrap::new(window)?;
        let physical_devices = enumerate_physical_devices(surface.vulkan_instance())?;
        let mut found_complete_device = false;

        println!(
            "M1-S6 checking queue/swapchain support for {} physical device(s)",
            physical_devices.len()
        );

        for physical_device in &physical_devices {
            let queue_support = query_queue_family_support(
                surface.vulkan_instance(),
                surface.surface_loader(),
                surface.surface(),
                physical_device.handle,
            )?;
            let swapchain_summary = query_swapchain_support_summary(
                surface.surface_loader(),
                physical_device.handle,
                surface.surface(),
            )?;

            found_complete_device |= queue_support.indices.is_complete();
            println!("  {}", physical_device.name);
            println!(
                "    graphics queue: {:?}, present queue: {:?}",
                queue_support.indices.graphics_family, queue_support.indices.present_family
            );
            println!(
                "    swapchain formats: {}, present modes: {}, image count: {}..{}",
                swapchain_summary.format_count,
                swapchain_summary.present_mode_count,
                swapchain_summary.min_image_count,
                if swapchain_summary.max_image_count == 0 {
                    u32::MAX
                } else {
                    swapchain_summary.max_image_count
                }
            );

            for report in &queue_support.reports {
                println!(
                    "    family {}: queues={}, flags={:?}, present={}",
                    report.family_index,
                    report.queue_count,
                    report.queue_flags,
                    report.supports_present
                );
            }
        }

        if !found_complete_device {
            return Err(DeviceError::NoSuitableQueueFamilies);
        }

        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: DeviceError) {
        eprintln!("M1-S6 queue/swapchain support query failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for QueueSupportShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_surface_and_report_support(event_loop) {
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
            println!("M1-S6 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.surface = None;
    }
}
