use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use ash::{Device as AshDevice, khr::swapchain::Device as SwapchainLoader, vk};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    DeviceError, LogicalDevice, SelectedPhysicalDevice, SurfaceBootstrap, VulkanInstanceError,
    WindowConfig, create_logical_device, select_physical_device,
};

#[derive(Debug)]
pub enum SwapchainError {
    Device(DeviceError),
    Vulkan(vk::Result),
    MissingSurfaceFormats,
    MissingPresentModes,
}

impl Display for SwapchainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Device(error) => write!(formatter, "device error: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
            Self::MissingSurfaceFormats => write!(formatter, "surface has no supported formats"),
            Self::MissingPresentModes => {
                write!(formatter, "surface has no supported present modes")
            }
        }
    }
}

impl Error for SwapchainError {}

impl From<DeviceError> for SwapchainError {
    fn from(error: DeviceError) -> Self {
        Self::Device(error)
    }
}

impl From<VulkanInstanceError> for SwapchainError {
    fn from(error: VulkanInstanceError) -> Self {
        Self::Device(DeviceError::from(error))
    }
}

impl From<vk::Result> for SwapchainError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

#[derive(Debug, Clone)]
pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Debug, Clone, Copy)]
pub struct SwapchainConfig {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
    pub image_count: u32,
}

pub struct SwapchainBundle {
    device: AshDevice,
    swapchain_loader: SwapchainLoader,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    config: SwapchainConfig,
}

impl Debug for SwapchainBundle {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SwapchainBundle")
            .field("swapchain", &self.swapchain)
            .field("image_count", &self.images.len())
            .field("image_view_count", &self.image_views.len())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl SwapchainBundle {
    pub fn swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn images(&self) -> &[vk::Image] {
        &self.images
    }

    pub fn image_views(&self) -> &[vk::ImageView] {
        &self.image_views
    }

    pub fn config(&self) -> SwapchainConfig {
        self.config
    }

    pub fn loader(&self) -> &SwapchainLoader {
        &self.swapchain_loader
    }
}

impl Drop for SwapchainBundle {
    fn drop(&mut self) {
        // SAFETY: image views and swapchain were created from this live device.
        // Image views are device child objects and must be destroyed before the swapchain bundle.
        unsafe {
            for image_view in self.image_views.drain(..) {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

pub fn query_swapchain_support_details(
    surface_loader: &ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<SwapchainSupportDetails, SwapchainError> {
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

    Ok(SwapchainSupportDetails {
        capabilities,
        formats,
        present_modes,
    })
}

pub fn choose_swapchain_config(
    details: &SwapchainSupportDetails,
    window_size: PhysicalSize<u32>,
) -> Result<SwapchainConfig, SwapchainError> {
    let surface_format = choose_surface_format(&details.formats)?;
    let present_mode = choose_present_mode(&details.present_modes)?;
    let extent = choose_extent(&details.capabilities, window_size);
    let image_count = choose_image_count(&details.capabilities);

    Ok(SwapchainConfig {
        surface_format,
        present_mode,
        extent,
        image_count,
    })
}

pub fn create_swapchain_bundle(
    surface: &SurfaceBootstrap,
    logical_device: &LogicalDevice,
    selected_device: &SelectedPhysicalDevice,
    window_size: PhysicalSize<u32>,
) -> Result<SwapchainBundle, SwapchainError> {
    let details = query_swapchain_support_details(
        surface.surface_loader(),
        selected_device.info.handle,
        surface.surface(),
    )?;
    let config = choose_swapchain_config(&details, window_size)?;
    let queue_family_indices = selected_device.queue_indices.unique_indices();
    let image_sharing_mode = if queue_family_indices.len() > 1 {
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };
    let mut create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface.surface())
        .min_image_count(config.image_count)
        .image_format(config.surface_format.format)
        .image_color_space(config.surface_format.color_space)
        .image_extent(config.extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .pre_transform(details.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(config.present_mode)
        .clipped(true);

    if queue_family_indices.len() > 1 {
        create_info = create_info.queue_family_indices(&queue_family_indices);
    }

    let swapchain_loader =
        SwapchainLoader::new(surface.vulkan_instance().handle(), logical_device.handle());

    // SAFETY: create info references data that lives for the call; surface/device/queues were
    // created from the same instance and selected physical device.
    let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };
    // SAFETY: swapchain belongs to this device and loader; ash owns the returned Vec.
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
    let mut image_views = Vec::with_capacity(images.len());

    for image in &images {
        let image_view_info = vk::ImageViewCreateInfo::default()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(config.surface_format.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        // SAFETY: each image belongs to this swapchain; image view info is valid for a 2D color view.
        let image_view = unsafe {
            logical_device
                .handle()
                .create_image_view(&image_view_info, None)?
        };
        image_views.push(image_view);
    }

    Ok(SwapchainBundle {
        device: logical_device.handle().clone(),
        swapchain_loader,
        swapchain,
        images,
        image_views,
        config,
    })
}

fn choose_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> Result<vk::SurfaceFormatKHR, SwapchainError> {
    if formats.is_empty() {
        return Err(SwapchainError::MissingSurfaceFormats);
    }

    Ok(formats
        .iter()
        .copied()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or(formats[0]))
}

fn choose_present_mode(
    present_modes: &[vk::PresentModeKHR],
) -> Result<vk::PresentModeKHR, SwapchainError> {
    if present_modes.is_empty() {
        return Err(SwapchainError::MissingPresentModes);
    }

    Ok(present_modes
        .iter()
        .copied()
        .find(|present_mode| *present_mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO))
}

fn choose_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    window_size: PhysicalSize<u32>,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    vk::Extent2D {
        width: window_size.width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: window_size.height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}

fn choose_image_count(capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
    let preferred_count = capabilities.min_image_count + 1;

    if capabilities.max_image_count > 0 {
        preferred_count.min(capabilities.max_image_count)
    } else {
        preferred_count
    }
}

pub fn run_swapchain_config_shell(config: WindowConfig) -> Result<(), SwapchainError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = SwapchainConfigShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)
        .map_err(DeviceError::from)?;
    app.result
}

pub fn run_swapchain_shell(config: WindowConfig) -> Result<(), SwapchainError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = SwapchainShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)
        .map_err(DeviceError::from)?;
    app.result
}

pub fn run_resizable_swapchain_shell(config: WindowConfig) -> Result<(), SwapchainError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = ResizableSwapchainShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)
        .map_err(DeviceError::from)?;
    app.result
}

#[derive(Debug)]
struct SwapchainConfigShell {
    config: WindowConfig,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), SwapchainError>,
}

impl SwapchainConfigShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_surface_and_choose_config(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), SwapchainError> {
        if self.surface.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(self.config.attributes())
                    .map_err(VulkanInstanceError::from)
                    .map_err(DeviceError::from)?,
            );
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before swapchain config query");
        let surface = SurfaceBootstrap::new(window).map_err(DeviceError::from)?;
        let selected_device = select_physical_device(
            surface.vulkan_instance(),
            surface.surface_loader(),
            surface.surface(),
        )?;
        let details = query_swapchain_support_details(
            surface.surface_loader(),
            selected_device.info.handle,
            surface.surface(),
        )?;
        let config = choose_swapchain_config(&details, window.inner_size())?;

        println!("M1-S9 selected device: {}", selected_device.info.name);
        println!(
            "M1-S9 swapchain config: format={:?}, color_space={:?}, present_mode={:?}, extent={}x{}, images={}",
            config.surface_format.format,
            config.surface_format.color_space,
            config.present_mode,
            config.extent.width,
            config.extent.height,
            config.image_count
        );

        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: SwapchainError) {
        eprintln!("M1-S9 swapchain config failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for SwapchainConfigShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_surface_and_choose_config(event_loop) {
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
            println!("M1-S9 window close requested");
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
struct SwapchainShell {
    config: WindowConfig,
    swapchain: Option<SwapchainBundle>,
    logical_device: Option<LogicalDevice>,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), SwapchainError>,
}

impl SwapchainShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            swapchain: None,
            logical_device: None,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_swapchain_resources(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), SwapchainError> {
        if self.swapchain.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(self.config.attributes())
                    .map_err(VulkanInstanceError::from)
                    .map_err(DeviceError::from)?,
            );
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before swapchain creation");
        let surface = SurfaceBootstrap::new(window).map_err(DeviceError::from)?;
        let selected_device = select_physical_device(
            surface.vulkan_instance(),
            surface.surface_loader(),
            surface.surface(),
        )?;
        let logical_device = create_logical_device(surface.vulkan_instance(), &selected_device)?;
        let swapchain = create_swapchain_bundle(
            &surface,
            &logical_device,
            &selected_device,
            window.inner_size(),
        )?;

        println!("M1-S10 selected device: {}", selected_device.info.name);
        println!(
            "M1-S10 swapchain created: {:?}, images={}, image_views={}, extent={}x{}",
            swapchain.swapchain(),
            swapchain.images().len(),
            swapchain.image_views().len(),
            swapchain.config().extent.width,
            swapchain.config().extent.height
        );

        self.swapchain = Some(swapchain);
        self.logical_device = Some(logical_device);
        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: SwapchainError) {
        eprintln!("M1-S10 swapchain creation failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for SwapchainShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_swapchain_resources(event_loop) {
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
            println!("M1-S10 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.swapchain = None;
        self.logical_device = None;
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.swapchain = None;
        self.logical_device = None;
        self.surface = None;
    }
}

#[derive(Debug)]
struct ResizableSwapchainShell {
    config: WindowConfig,
    swapchain: Option<SwapchainBundle>,
    logical_device: Option<LogicalDevice>,
    selected_device: Option<SelectedPhysicalDevice>,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    resize_pending: bool,
    result: Result<(), SwapchainError>,
}

impl ResizableSwapchainShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            swapchain: None,
            logical_device: None,
            selected_device: None,
            surface: None,
            window: None,
            resize_pending: false,
            result: Ok(()),
        }
    }

    fn create_initial_resources(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), SwapchainError> {
        if self.swapchain.is_some() {
            return Ok(());
        }

        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(self.config.attributes())
                    .map_err(VulkanInstanceError::from)
                    .map_err(DeviceError::from)?,
            );
        }

        let window = self
            .window
            .as_ref()
            .expect("window was created before resizable swapchain setup");
        let surface = SurfaceBootstrap::new(window).map_err(DeviceError::from)?;
        let selected_device = select_physical_device(
            surface.vulkan_instance(),
            surface.surface_loader(),
            surface.surface(),
        )?;
        let logical_device = create_logical_device(surface.vulkan_instance(), &selected_device)?;
        let swapchain = create_swapchain_bundle(
            &surface,
            &logical_device,
            &selected_device,
            window.inner_size(),
        )?;

        println!(
            "M1-S11 initial swapchain: images={}, image_views={}, extent={}x{}",
            swapchain.images().len(),
            swapchain.image_views().len(),
            swapchain.config().extent.width,
            swapchain.config().extent.height
        );

        self.swapchain = Some(swapchain);
        self.logical_device = Some(logical_device);
        self.selected_device = Some(selected_device);
        self.surface = Some(surface);

        Ok(())
    }

    fn recreate_swapchain(&mut self) -> Result<(), SwapchainError> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();

        if size.width == 0 || size.height == 0 {
            return Ok(());
        }

        let logical_device = self
            .logical_device
            .as_ref()
            .expect("logical device exists before swapchain recreate");
        let surface = self
            .surface
            .as_ref()
            .expect("surface exists before swapchain recreate");
        let selected_device = self
            .selected_device
            .as_ref()
            .expect("selected device exists before swapchain recreate");

        logical_device.wait_idle().map_err(SwapchainError::from)?;
        self.swapchain = None;
        let swapchain = create_swapchain_bundle(surface, logical_device, selected_device, size)?;

        println!(
            "M1-S11 recreated swapchain: images={}, image_views={}, extent={}x{}",
            swapchain.images().len(),
            swapchain.image_views().len(),
            swapchain.config().extent.width,
            swapchain.config().extent.height
        );

        self.swapchain = Some(swapchain);
        self.resize_pending = false;

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: SwapchainError) {
        eprintln!("M1-S11 resize/recreate failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for ResizableSwapchainShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_initial_resources(event_loop) {
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

        match event {
            WindowEvent::CloseRequested => {
                println!("M1-S11 window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize_pending = true;
                println!("M1-S11 resize requested: {}x{}", size.width, size.height);

                if let Err(error) = self.recreate_swapchain() {
                    self.record_error_and_exit(event_loop, error);
                }
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.swapchain = None;
        self.logical_device = None;
        self.selected_device = None;
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.swapchain = None;
        self.logical_device = None;
        self.selected_device = None;
        self.surface = None;
    }
}
