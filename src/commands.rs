use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use ash::{Device as AshDevice, vk};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    DeviceError, LogicalDevice, SelectedPhysicalDevice, SurfaceBootstrap, SwapchainBundle,
    SwapchainError, VulkanInstanceError, WindowConfig, create_logical_device,
    create_swapchain_bundle, select_physical_device,
};

#[derive(Debug)]
pub enum CommandError {
    Device(DeviceError),
    Swapchain(SwapchainError),
    Vulkan(vk::Result),
}

impl Display for CommandError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Device(error) => write!(formatter, "device error: {error}"),
            Self::Swapchain(error) => write!(formatter, "swapchain error: {error}"),
            Self::Vulkan(error) => write!(formatter, "Vulkan error: {error:?}"),
        }
    }
}

impl Error for CommandError {}

impl From<DeviceError> for CommandError {
    fn from(error: DeviceError) -> Self {
        Self::Device(error)
    }
}

impl From<VulkanInstanceError> for CommandError {
    fn from(error: VulkanInstanceError) -> Self {
        Self::Device(DeviceError::from(error))
    }
}

impl From<SwapchainError> for CommandError {
    fn from(error: SwapchainError) -> Self {
        Self::Swapchain(error)
    }
}

impl From<vk::Result> for CommandError {
    fn from(error: vk::Result) -> Self {
        Self::Vulkan(error)
    }
}

pub struct ClearCommandBundle {
    device: AshDevice,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    clear_color: vk::ClearColorValue,
}

pub struct FrameSync {
    device: AshDevice,
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,
}

impl Debug for FrameSync {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrameSync")
            .field("image_available", &self.image_available)
            .field("render_finished", &self.render_finished)
            .field("in_flight", &self.in_flight)
            .finish()
    }
}

impl FrameSync {
    pub fn new(logical_device: &LogicalDevice) -> Result<Self, CommandError> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        // SAFETY: synchronization objects are created from a live logical device.
        let image_available = unsafe {
            logical_device
                .handle()
                .create_semaphore(&semaphore_info, None)?
        };
        // SAFETY: synchronization objects are created from a live logical device.
        let render_finished = unsafe {
            logical_device
                .handle()
                .create_semaphore(&semaphore_info, None)?
        };
        // SAFETY: synchronization objects are created from a live logical device.
        let in_flight = unsafe { logical_device.handle().create_fence(&fence_info, None)? };

        Ok(Self {
            device: logical_device.handle().clone(),
            image_available,
            render_finished,
            in_flight,
        })
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        // SAFETY: sync objects were created from this live device and are not destroyed elsewhere.
        unsafe {
            self.device.destroy_fence(self.in_flight, None);
            self.device.destroy_semaphore(self.render_finished, None);
            self.device.destroy_semaphore(self.image_available, None);
        }
    }
}

impl Debug for ClearCommandBundle {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClearCommandBundle")
            .field("command_pool", &self.command_pool)
            .field("command_buffer_count", &self.command_buffers.len())
            .finish_non_exhaustive()
    }
}

impl ClearCommandBundle {
    pub fn command_buffers(&self) -> &[vk::CommandBuffer] {
        &self.command_buffers
    }

    pub fn clear_color(&self) -> vk::ClearColorValue {
        self.clear_color
    }
}

impl Drop for ClearCommandBundle {
    fn drop(&mut self) {
        // SAFETY: command pool was created from this live device; destroying the pool frees
        // all command buffers allocated from it.
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}

pub fn create_clear_command_bundle(
    logical_device: &LogicalDevice,
    swapchain: &SwapchainBundle,
    clear_color: vk::ClearColorValue,
) -> Result<ClearCommandBundle, CommandError> {
    let graphics_family = logical_device
        .queue_indices()
        .graphics_family
        .expect("logical device has graphics queue family");
    let command_pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(graphics_family);

    // SAFETY: queue family index belongs to the live logical device.
    let command_pool = unsafe {
        logical_device
            .handle()
            .create_command_pool(&command_pool_info, None)?
    };
    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(swapchain.images().len() as u32);

    // SAFETY: command pool is live and allocation count matches swapchain image count.
    let command_buffers = unsafe {
        logical_device
            .handle()
            .allocate_command_buffers(&allocate_info)?
    };
    let bundle = ClearCommandBundle {
        device: logical_device.handle().clone(),
        command_pool,
        command_buffers,
        clear_color,
    };

    record_clear_commands(logical_device, swapchain, &bundle)?;

    Ok(bundle)
}

pub fn draw_clear_frame(
    logical_device: &LogicalDevice,
    swapchain: &SwapchainBundle,
    commands: &ClearCommandBundle,
    sync: &FrameSync,
) -> Result<bool, CommandError> {
    // SAFETY: fence belongs to this live device and synchronizes the single in-flight frame.
    unsafe {
        logical_device
            .handle()
            .wait_for_fences(&[sync.in_flight], true, u64::MAX)?;
    }

    let (image_index, acquire_suboptimal) = match unsafe {
        swapchain.loader().acquire_next_image(
            swapchain.swapchain(),
            u64::MAX,
            sync.image_available,
            vk::Fence::null(),
        )
    } {
        Ok(result) => result,
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Ok(true),
        Err(error) => return Err(CommandError::from(error)),
    };

    // SAFETY: the previous frame has completed, so the fence can be reused for this submit.
    unsafe {
        logical_device.handle().reset_fences(&[sync.in_flight])?;
    }

    let wait_semaphores = [sync.image_available];
    let wait_stages = [vk::PipelineStageFlags::TRANSFER];
    let command_buffers = [commands.command_buffers()[image_index as usize]];
    let signal_semaphores = [sync.render_finished];
    let submit_info = vk::SubmitInfo::default()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&wait_stages)
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphores);

    // SAFETY: command buffer at `image_index` was recorded for that swapchain image.
    unsafe {
        logical_device.handle().queue_submit(
            logical_device.graphics_queue(),
            &[submit_info],
            sync.in_flight,
        )?;
    }

    let swapchains = [swapchain.swapchain()];
    let image_indices = [image_index];
    let present_info = vk::PresentInfoKHR::default()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&swapchains)
        .image_indices(&image_indices);
    let present_suboptimal = match unsafe {
        swapchain
            .loader()
            .queue_present(logical_device.present_queue(), &present_info)
    } {
        Ok(suboptimal) => suboptimal,
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,
        Err(error) => return Err(CommandError::from(error)),
    };

    Ok(acquire_suboptimal || present_suboptimal)
}

fn record_clear_commands(
    logical_device: &LogicalDevice,
    swapchain: &SwapchainBundle,
    commands: &ClearCommandBundle,
) -> Result<(), CommandError> {
    let subresource_range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };

    for (command_buffer, image) in commands
        .command_buffers
        .iter()
        .copied()
        .zip(swapchain.images().iter().copied())
    {
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        // SAFETY: command buffer was allocated from a resettable pool and is not recording.
        unsafe {
            logical_device
                .handle()
                .begin_command_buffer(command_buffer, &begin_info)?;
        }

        transition_image_layout(
            logical_device.handle(),
            command_buffer,
            image,
            subresource_range,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        );

        // SAFETY: image is in TRANSFER_DST_OPTIMAL for the recorded command buffer.
        unsafe {
            logical_device.handle().cmd_clear_color_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &commands.clear_color,
                &[subresource_range],
            );
        }

        transition_image_layout(
            logical_device.handle(),
            command_buffer,
            image,
            subresource_range,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::empty(),
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
        );

        // SAFETY: all commands were recorded into this command buffer and recording is active.
        unsafe {
            logical_device.handle().end_command_buffer(command_buffer)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn transition_image_layout(
    device: &AshDevice,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    subresource_range: vk::ImageSubresourceRange,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_access_mask: vk::AccessFlags,
    dst_access_mask: vk::AccessFlags,
    src_stage: vk::PipelineStageFlags,
    dst_stage: vk::PipelineStageFlags,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource_range)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    // SAFETY: barrier describes a swapchain image used by the command buffer's graphics queue.
    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}

pub fn run_clear_command_shell(config: WindowConfig) -> Result<(), CommandError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = ClearCommandShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)
        .map_err(DeviceError::from)?;
    app.result
}

pub fn run_present_loop_shell(config: WindowConfig) -> Result<(), CommandError> {
    let event_loop = EventLoop::new().map_err(VulkanInstanceError::from)?;
    let mut app = PresentLoopShell::new(config);

    event_loop
        .run_app(&mut app)
        .map_err(VulkanInstanceError::from)
        .map_err(DeviceError::from)?;
    app.result
}

#[derive(Debug)]
struct ClearCommandShell {
    config: WindowConfig,
    commands: Option<ClearCommandBundle>,
    swapchain: Option<SwapchainBundle>,
    logical_device: Option<LogicalDevice>,
    selected_device: Option<SelectedPhysicalDevice>,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    result: Result<(), CommandError>,
}

impl ClearCommandShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            commands: None,
            swapchain: None,
            logical_device: None,
            selected_device: None,
            surface: None,
            window: None,
            result: Ok(()),
        }
    }

    fn create_and_record_commands(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), CommandError> {
        if self.commands.is_some() {
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
            .expect("window was created before clear command recording");
        let surface = SurfaceBootstrap::new(window)
            .map_err(DeviceError::from)
            .map_err(SwapchainError::from)?;
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
        let clear_color = vk::ClearColorValue {
            float32: [0.02, 0.08, 0.16, 1.0],
        };
        let commands = create_clear_command_bundle(&logical_device, &swapchain, clear_color)?;

        println!(
            "M1-S12 recorded {} clear command buffer(s) for swapchain {:?}",
            commands.command_buffers().len(),
            swapchain.swapchain()
        );

        self.commands = Some(commands);
        self.swapchain = Some(swapchain);
        self.logical_device = Some(logical_device);
        self.selected_device = Some(selected_device);
        self.surface = Some(surface);

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: CommandError) {
        eprintln!("M1-S12 clear command recording failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for ClearCommandShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_and_record_commands(event_loop) {
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
            println!("M1-S12 window close requested");
            event_loop.exit();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.commands = None;
        self.swapchain = None;
        self.logical_device = None;
        self.selected_device = None;
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.commands = None;
        self.swapchain = None;
        self.logical_device = None;
        self.selected_device = None;
        self.surface = None;
    }
}

#[derive(Debug)]
struct PresentLoopShell {
    config: WindowConfig,
    sync: Option<FrameSync>,
    commands: Option<ClearCommandBundle>,
    swapchain: Option<SwapchainBundle>,
    logical_device: Option<LogicalDevice>,
    selected_device: Option<SelectedPhysicalDevice>,
    surface: Option<SurfaceBootstrap>,
    window: Option<Window>,
    framebuffer_resized: bool,
    result: Result<(), CommandError>,
}

impl PresentLoopShell {
    fn new(config: WindowConfig) -> Self {
        Self {
            config,
            sync: None,
            commands: None,
            swapchain: None,
            logical_device: None,
            selected_device: None,
            surface: None,
            window: None,
            framebuffer_resized: false,
            result: Ok(()),
        }
    }

    fn create_resources(&mut self, event_loop: &ActiveEventLoop) -> Result<(), CommandError> {
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
            .expect("window was created before present loop setup");
        let surface = SurfaceBootstrap::new(window)
            .map_err(DeviceError::from)
            .map_err(SwapchainError::from)?;
        let selected_device = select_physical_device(
            surface.vulkan_instance(),
            surface.surface_loader(),
            surface.surface(),
        )?;
        let logical_device = create_logical_device(surface.vulkan_instance(), &selected_device)?;
        let sync = FrameSync::new(&logical_device)?;
        let swapchain = create_swapchain_bundle(
            &surface,
            &logical_device,
            &selected_device,
            window.inner_size(),
        )?;
        let clear_color = vk::ClearColorValue {
            float32: [0.02, 0.08, 0.16, 1.0],
        };
        let commands = create_clear_command_bundle(&logical_device, &swapchain, clear_color)?;

        println!(
            "M1-S13 present loop ready: images={}, extent={}x{}",
            swapchain.images().len(),
            swapchain.config().extent.width,
            swapchain.config().extent.height
        );

        self.sync = Some(sync);
        self.commands = Some(commands);
        self.swapchain = Some(swapchain);
        self.logical_device = Some(logical_device);
        self.selected_device = Some(selected_device);
        self.surface = Some(surface);

        Ok(())
    }

    fn recreate_swapchain_and_commands(&mut self) -> Result<(), CommandError> {
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
            .expect("logical device exists before present loop recreate");
        let surface = self
            .surface
            .as_ref()
            .expect("surface exists before present loop recreate");
        let selected_device = self
            .selected_device
            .as_ref()
            .expect("selected device exists before present loop recreate");

        logical_device.wait_idle()?;
        self.commands = None;
        self.swapchain = None;
        let swapchain = create_swapchain_bundle(surface, logical_device, selected_device, size)?;
        let clear_color = vk::ClearColorValue {
            float32: [0.02, 0.08, 0.16, 1.0],
        };
        let commands = create_clear_command_bundle(logical_device, &swapchain, clear_color)?;

        println!(
            "M1-S13 recreated present resources: images={}, extent={}x{}",
            swapchain.images().len(),
            swapchain.config().extent.width,
            swapchain.config().extent.height
        );

        self.commands = Some(commands);
        self.swapchain = Some(swapchain);
        self.framebuffer_resized = false;

        Ok(())
    }

    fn draw_frame(&mut self) -> Result<(), CommandError> {
        if self.framebuffer_resized {
            self.recreate_swapchain_and_commands()?;
            return Ok(());
        }

        let Some(logical_device) = &self.logical_device else {
            return Ok(());
        };
        let Some(swapchain) = &self.swapchain else {
            return Ok(());
        };
        let Some(commands) = &self.commands else {
            return Ok(());
        };
        let Some(sync) = &self.sync else {
            return Ok(());
        };

        if draw_clear_frame(logical_device, swapchain, commands, sync)? {
            self.recreate_swapchain_and_commands()?;
        }

        Ok(())
    }

    fn record_error_and_exit(&mut self, event_loop: &ActiveEventLoop, error: CommandError) {
        eprintln!("M1-S13 present loop failed: {error}");
        self.result = Err(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for PresentLoopShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.create_resources(event_loop) {
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
                println!("M1-S13 window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.framebuffer_resized = true;
                println!("M1-S13 resize requested: {}x{}", size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw_frame() {
                    self.record_error_and_exit(event_loop, error);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(logical_device) = &self.logical_device
            && let Err(error) = logical_device.wait_idle()
        {
            self.result = Err(CommandError::from(error));
        }

        self.sync = None;
        self.commands = None;
        self.swapchain = None;
        self.logical_device = None;
        self.selected_device = None;
        self.surface = None;
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(logical_device) = &self.logical_device
            && let Err(error) = logical_device.wait_idle()
        {
            self.result = Err(CommandError::from(error));
        }

        self.sync = None;
        self.commands = None;
        self.swapchain = None;
        self.logical_device = None;
        self.selected_device = None;
        self.surface = None;
    }
}
