use std::{
    error::Error,
    ffi::CStr,
    fmt::{Debug, Display, Formatter},
};

use ash::{Device as AshDevice, vk};
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
    NoSuitablePhysicalDevice,
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
            Self::NoSuitablePhysicalDevice => {
                write!(
                    formatter,
                    "no physical device satisfies renderer requirements"
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

    pub fn unique_indices(&self) -> Vec<u32> {
        let mut indices = Vec::with_capacity(2);

        if let Some(graphics_family) = self.graphics_family {
            indices.push(graphics_family);
        }

        if let Some(present_family) = self.present_family
            && !indices.contains(&present_family)
        {
            indices.push(present_family);
        }

        indices
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
pub struct DeviceExtensionSupport {
    pub swapchain: bool,
    pub acceleration_structure: bool,
    pub ray_tracing_pipeline: bool,
    pub deferred_host_operations: bool,
    pub buffer_device_address: bool,
    pub spirv_1_4: bool,
    pub shader_float_controls: bool,
}

#[derive(Debug, Clone)]
pub struct RayTracingFeatureSupport {
    pub acceleration_structure: bool,
    pub ray_tracing_pipeline: bool,
    pub ray_traversal_primitive_culling: bool,
    pub buffer_device_address: bool,
}

#[derive(Debug, Clone)]
pub struct PhysicalDeviceFeatureMatrix {
    pub extensions: DeviceExtensionSupport,
    pub ray_tracing_features: RayTracingFeatureSupport,
}

#[derive(Debug, Clone)]
pub struct SelectedPhysicalDevice {
    pub info: PhysicalDeviceInfo,
    pub queue_indices: QueueFamilyIndices,
    pub swapchain_support: SwapchainSupportSummary,
    pub feature_matrix: PhysicalDeviceFeatureMatrix,
}

pub struct LogicalDevice {
    device: AshDevice,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    queue_indices: QueueFamilyIndices,
}

impl Debug for LogicalDevice {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LogicalDevice")
            .field("graphics_queue", &self.graphics_queue)
            .field("present_queue", &self.present_queue)
            .field("queue_indices", &self.queue_indices)
            .finish_non_exhaustive()
    }
}

impl LogicalDevice {
    pub fn handle(&self) -> &AshDevice {
        &self.device
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn queue_indices(&self) -> QueueFamilyIndices {
        self.queue_indices
    }

    pub fn wait_idle(&self) -> Result<(), DeviceError> {
        // SAFETY: waiting idle is valid for a live logical device and synchronizes all queues.
        unsafe {
            self.device.device_wait_idle()?;
        }

        Ok(())
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        // SAFETY: `LogicalDevice` 是 `VkDevice` 的唯一 owner。
        // 后续 device child objects 必须在这个 owner drop 之前销毁。
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

impl PhysicalDeviceFeatureMatrix {
    pub fn swapchain_ready(&self) -> bool {
        self.extensions.swapchain
    }

    pub fn ray_tracing_ready(&self) -> bool {
        self.extensions.acceleration_structure
            && self.extensions.ray_tracing_pipeline
            && self.extensions.deferred_host_operations
            && self.extensions.buffer_device_address
            && self.extensions.spirv_1_4
            && self.extensions.shader_float_controls
            && self.ray_tracing_features.acceleration_structure
            && self.ray_tracing_features.ray_tracing_pipeline
            && self.ray_tracing_features.buffer_device_address
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

pub fn query_physical_device_feature_matrix(
    vulkan_instance: &VulkanInstance,
    physical_device: vk::PhysicalDevice,
) -> Result<PhysicalDeviceFeatureMatrix, DeviceError> {
    // SAFETY: `physical_device` was returned by this live instance; ash owns the output Vec.
    let extension_properties = unsafe {
        vulkan_instance
            .handle()
            .enumerate_device_extension_properties(physical_device)?
    };
    let extensions = DeviceExtensionSupport {
        swapchain: device_extension_available(&extension_properties, ash::khr::swapchain::NAME),
        acceleration_structure: device_extension_available(
            &extension_properties,
            ash::khr::acceleration_structure::NAME,
        ),
        ray_tracing_pipeline: device_extension_available(
            &extension_properties,
            ash::khr::ray_tracing_pipeline::NAME,
        ),
        deferred_host_operations: device_extension_available(
            &extension_properties,
            ash::khr::deferred_host_operations::NAME,
        ),
        buffer_device_address: device_extension_available(
            &extension_properties,
            ash::khr::buffer_device_address::NAME,
        ),
        spirv_1_4: device_extension_available(&extension_properties, ash::khr::spirv_1_4::NAME),
        shader_float_controls: device_extension_available(
            &extension_properties,
            ash::khr::shader_float_controls::NAME,
        ),
    };

    let mut acceleration_structure_features =
        vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
    let mut ray_tracing_pipeline_features =
        vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default();
    let mut buffer_device_address_features =
        vk::PhysicalDeviceBufferDeviceAddressFeatures::default();
    let mut features = vk::PhysicalDeviceFeatures2::default()
        .push_next(&mut acceleration_structure_features)
        .push_next(&mut ray_tracing_pipeline_features)
        .push_next(&mut buffer_device_address_features);

    // SAFETY: the feature structs are valid `pNext` chain nodes and live for the duration
    // of this call; `physical_device` belongs to this instance.
    unsafe {
        vulkan_instance
            .handle()
            .get_physical_device_features2(physical_device, &mut features);
    }

    Ok(PhysicalDeviceFeatureMatrix {
        extensions,
        ray_tracing_features: RayTracingFeatureSupport {
            acceleration_structure: acceleration_structure_features.acceleration_structure
                == vk::TRUE,
            ray_tracing_pipeline: ray_tracing_pipeline_features.ray_tracing_pipeline == vk::TRUE,
            ray_traversal_primitive_culling: ray_tracing_pipeline_features
                .ray_traversal_primitive_culling
                == vk::TRUE,
            buffer_device_address: buffer_device_address_features.buffer_device_address == vk::TRUE,
        },
    })
}

pub fn select_physical_device(
    vulkan_instance: &VulkanInstance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<SelectedPhysicalDevice, DeviceError> {
    let physical_devices = enumerate_physical_devices(vulkan_instance)?;

    for info in physical_devices {
        let queue_support =
            query_queue_family_support(vulkan_instance, surface_loader, surface, info.handle)?;

        if !queue_support.indices.is_complete() {
            continue;
        }

        let feature_matrix = query_physical_device_feature_matrix(vulkan_instance, info.handle)?;

        if !feature_matrix.swapchain_ready() {
            continue;
        }

        let swapchain_support =
            query_swapchain_support_summary(surface_loader, info.handle, surface)?;

        if swapchain_support.format_count == 0 || swapchain_support.present_mode_count == 0 {
            continue;
        }

        return Ok(SelectedPhysicalDevice {
            info,
            queue_indices: queue_support.indices,
            swapchain_support,
            feature_matrix,
        });
    }

    Err(DeviceError::NoSuitablePhysicalDevice)
}

pub fn create_logical_device(
    vulkan_instance: &VulkanInstance,
    selected_device: &SelectedPhysicalDevice,
) -> Result<LogicalDevice, DeviceError> {
    let queue_priority = [1.0_f32];
    let unique_queue_indices = selected_device.queue_indices.unique_indices();
    let queue_infos = unique_queue_indices
        .iter()
        .map(|queue_family_index| {
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(*queue_family_index)
                .queue_priorities(&queue_priority)
        })
        .collect::<Vec<_>>();
    let device_extension_names = [ash::khr::swapchain::NAME.as_ptr()];
    let device_features = vk::PhysicalDeviceFeatures::default();
    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_names)
        .enabled_features(&device_features);

    // SAFETY: selected device and queue family indices were queried from this live instance.
    // The queue info and extension pointer arrays live for the duration of the call.
    let device = unsafe {
        vulkan_instance
            .handle()
            .create_device(selected_device.info.handle, &device_info, None)?
    };
    let graphics_family = selected_device
        .queue_indices
        .graphics_family
        .expect("selected device has graphics queue family");
    let present_family = selected_device
        .queue_indices
        .present_family
        .expect("selected device has present queue family");

    // SAFETY: queues at index 0 exist because each queue create info requested one queue.
    let graphics_queue = unsafe { device.get_device_queue(graphics_family, 0) };
    // SAFETY: queues at index 0 exist because each queue create info requested one queue.
    let present_queue = unsafe { device.get_device_queue(present_family, 0) };

    Ok(LogicalDevice {
        device,
        graphics_queue,
        present_queue,
        queue_indices: selected_device.queue_indices,
    })
}

fn device_extension_available(
    extension_properties: &[vk::ExtensionProperties],
    required_extension: &CStr,
) -> bool {
    extension_properties.iter().any(|extension| {
        // SAFETY: Vulkan guarantees `extension_name` is a nul-terminated fixed-size C string.
        let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };
        extension_name == required_extension
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

pub fn run_feature_matrix_shell(config: WindowConfig) -> Result<(), DeviceError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = FeatureMatrixShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)?;
    app.result
}

pub fn run_logical_device_shell(config: WindowConfig) -> Result<(), DeviceError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = LogicalDeviceShell::new(config);

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

#[derive(Debug)]
struct FeatureMatrixShell {
    config: WindowConfig,
    vulkan_instance: Option<VulkanInstance>,
    window: Option<Window>,
    result: Result<(), DeviceError>,
}

impl FeatureMatrixShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            vulkan_instance: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_instance_and_report_features(
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
            .expect("window was created before feature matrix query");
        let display_handle = window
            .display_handle()
            .map_err(VulkanInstanceError::from)?
            .as_raw();
        let vulkan_instance =
            VulkanInstance::new_for_display(display_handle, "learn-vulkan-renderer-m1-s7")?;
        let physical_devices = enumerate_physical_devices(&vulkan_instance)?;

        println!(
            "M1-S7 checking extension/feature matrix for {} physical device(s)",
            physical_devices.len()
        );

        for physical_device in &physical_devices {
            let matrix =
                query_physical_device_feature_matrix(&vulkan_instance, physical_device.handle)?;

            println!("  {}", physical_device.name);
            println!(
                "    required swapchain extension: {}",
                matrix.extensions.swapchain
            );
            println!(
                "    ray tracing extensions: as={}, rtp={}, deferred_host={}, bda={}, spirv14={}, shader_float_controls={}",
                matrix.extensions.acceleration_structure,
                matrix.extensions.ray_tracing_pipeline,
                matrix.extensions.deferred_host_operations,
                matrix.extensions.buffer_device_address,
                matrix.extensions.spirv_1_4,
                matrix.extensions.shader_float_controls
            );
            println!(
                "    ray tracing features: as={}, rtp={}, culling={}, bda={}",
                matrix.ray_tracing_features.acceleration_structure,
                matrix.ray_tracing_features.ray_tracing_pipeline,
                matrix.ray_tracing_features.ray_traversal_primitive_culling,
                matrix.ray_tracing_features.buffer_device_address
            );
            println!(
                "    readiness: swapchain={}, ray_tracing={}",
                matrix.swapchain_ready(),
                matrix.ray_tracing_ready()
            );
        }

        self.vulkan_instance = Some(vulkan_instance);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: DeviceError) {
        eprintln!("M1-S7 feature matrix query failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for FeatureMatrixShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_instance_and_report_features(event_loop) {
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
            println!("M1-S7 window close requested");
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
struct LogicalDeviceShell {
    config: WindowConfig,
    logical_device: Option<LogicalDevice>,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), DeviceError>,
}

impl LogicalDeviceShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            logical_device: None,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_surface_and_logical_device(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), DeviceError> {
        if self.logical_device.is_some() {
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
            .expect("window was created before logical device bootstrap");
        let surface = SurfaceBootstrap::new(window)?;
        let selected_device = select_physical_device(
            surface.vulkan_instance(),
            surface.surface_loader(),
            surface.surface(),
        )?;
        let logical_device = create_logical_device(surface.vulkan_instance(), &selected_device)?;

        println!("M1-S8 selected device: {}", selected_device.info);
        println!(
            "M1-S8 logical device created; graphics queue {:?}, present queue {:?}",
            logical_device.graphics_queue(),
            logical_device.present_queue()
        );
        println!(
            "M1-S8 queue families: graphics={:?}, present={:?}",
            logical_device.queue_indices().graphics_family,
            logical_device.queue_indices().present_family
        );

        self.logical_device = Some(logical_device);
        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: DeviceError) {
        eprintln!("M1-S8 logical device bootstrap failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for LogicalDeviceShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_surface_and_logical_device(event_loop) {
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
            println!("M1-S8 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.logical_device = None;
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.logical_device = None;
        self.surface = None;
    }
}
