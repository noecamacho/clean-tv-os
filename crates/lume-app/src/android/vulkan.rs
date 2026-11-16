//! The Vulkan 1.3 renderer: the same frames as `Gles2Backend`, drawn with dynamic rendering,
//! push descriptors and incremental present.
//!
//! One frame is in flight at a time: everything a frame writes (vertices, the staging buffer of
//! texture uploads) is reused once its fence has signaled. Texture uploads between two frames are
//! recorded in their own command buffer and submitted with the next frame.

use ash::vk;
use lume_core::render::{
    cover_size, Frame, Gles2Backend, Program, RectVertex, TexVertex, Texture as ImageTexture,
    Viewport, BAYER_4X4, MAX_QUADS_PER_BATCH, QUAD_INDICES,
};
use lume_core::text::{TextEngine, Update};
use lume_core::theme::Color;
use lume_core::wallpaper::{self, WallpaperId};
use std::ffi::c_void;
use std::mem::size_of;
use std::path::PathBuf;

/// Rows of the wallpaper sent to the GPU per frame while it replaces another one.
const WALLPAPER_UPLOAD_ROWS: u32 = 90;
/// Texture bytes that can wait for one submit: a whole 1920x1080 RGBA wallpaper fits.
const STAGING_BYTES: u64 = 10 << 20;
const RGBA8: vk::Format = vk::Format::R8G8B8A8_UNORM;
const READ: vk::ImageLayout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;

const PIPELINES: usize = 6;
const RECT: usize = 0;
const TEXT: usize = 1;
const IMAGE: usize = 2;
const GLASS: usize = 3;
const WALLPAPER: usize = 4;
const REDUCE: usize = 5;

#[derive(Clone, Copy)]
struct Tex {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    width: u32,
    height: u32,
}

/// A buffer the CPU writes through `ptr` for as long as it lives.
struct Buf {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    ptr: *mut u8,
    size: u64,
}

impl Buf {
    const NULL: Buf = Buf {
        buffer: vk::Buffer::null(),
        memory: vk::DeviceMemory::null(),
        ptr: std::ptr::null_mut(),
        size: 0,
    };
}

struct SwapImage {
    image: vk::Image,
    view: vk::ImageView,
    /// Signaled by the frame drawn on this image, waited for by its present.
    drawn: vk::Semaphore,
    /// The number of the frame last presented on it; `None` while its content is undefined.
    presented: Option<u64>,
}

struct WallpaperUpload {
    id: WallpaperId,
    cover: (u32, u32),
    rgb: Vec<u8>,
    rows_done: u32,
    source: Tex,
    started: std::time::Instant,
    loaded: std::time::Duration,
}

pub struct VkRenderer {
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_fn: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical: vk::PhysicalDevice,
    memory: vk::PhysicalDeviceMemoryProperties,
    device: ash::Device,
    queue: vk::Queue,
    swap_fn: ash::khr::swapchain::Device,
    push_fn: ash::khr::push_descriptor::Device,
    gpu: (String, String),

    pool: vk::CommandPool,
    frame_cmd: vk::CommandBuffer,
    upload_cmd: vk::CommandBuffer,
    /// Whether `upload_cmd` is being recorded (it goes with the next frame).
    uploading_cmd: bool,
    fence: vk::Fence,
    /// Whether a submit is pending on `fence`.
    submitted: bool,
    acquire: vk::Semaphore,

    swapchain: vk::SwapchainKHR,
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<SwapImage>,
    acquired: Option<u32>,
    frame_number: u64,
    /// The surface changed: the swapchain is made again before the next frame.
    stale: bool,

    set_layout: vk::DescriptorSetLayout,
    layout: vk::PipelineLayout,
    pipelines: [vk::Pipeline; PIPELINES],
    linear: vk::Sampler,
    nearest_repeat: vk::Sampler,

    staging: Buf,
    staging_used: u64,
    rect_vertices: Buf,
    tex_vertices: Buf,
    indices: Buf,
    /// Textures replaced while a submit that may still read them is pending.
    garbage: Vec<Tex>,

    /// What a program samples when its texture does not exist yet.
    blank: Option<Tex>,
    atlas: Option<Tex>,
    image: Option<Tex>,
    wallpaper: Option<Tex>,
    wallpaper_id: Option<WallpaperId>,
    wallpaper_blur: Option<Tex>,
    wallpaper_cover: (u32, u32),
    wallpaper_thumbs: Option<Tex>,
    bayer: Option<Tex>,
    render_size: (u32, u32),
    async_wallpaper: bool,
    pending_wallpaper: Option<(WallpaperId, std::sync::mpsc::Receiver<Option<Vec<u8>>>)>,
    uploading: Option<WallpaperUpload>,
    wallpaper_failed: Option<WallpaperId>,
    content_changed: bool,
    cache_file: Option<PathBuf>,
}

fn err<T>(what: &str) -> impl FnOnce(vk::Result) -> Result<T, String> + '_ {
    move |e| Err(format!("{what}: {e}"))
}

impl VkRenderer {
    /// Creates the device, the swapchain of `window` (an `ANativeWindow`) at `size` and the
    /// pipelines. Fails without leaving anything on the window when the TV has no Vulkan 1.3
    /// with the extensions used, so the caller can fall back to OpenGL ES.
    pub fn new(
        window: *mut c_void,
        size: (u32, u32),
        cache_dir: Option<PathBuf>,
    ) -> Result<VkRenderer, String> {
        let started = std::time::Instant::now();
        // SAFETY: loads the system's libvulkan; every call below follows the Vulkan API's rules,
        // with handles this function creates.
        unsafe {
            let entry = ash::Entry::load().map_err(|e| format!("cannot load libvulkan: {e}"))?;
            let version = entry
                .try_enumerate_instance_version()
                .ok()
                .flatten()
                .unwrap_or(vk::API_VERSION_1_0);
            if version < vk::API_VERSION_1_3 {
                return Err("the Vulkan loader is older than 1.3".into());
            }
            let app = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);
            let extensions = [
                ash::khr::surface::NAME.as_ptr(),
                ash::khr::android_surface::NAME.as_ptr(),
            ];
            let instance = entry
                .create_instance(
                    &vk::InstanceCreateInfo::default()
                        .application_info(&app)
                        .enabled_extension_names(&extensions),
                    None,
                )
                .or_else(err("vkCreateInstance"))?;
            let surface_fn = ash::khr::surface::Instance::new(&entry, &instance);
            let made = Self::open_device(&entry, &instance, &surface_fn, window);
            let (surface, physical, family, device, gpu) = match made {
                Ok(made) => made,
                Err(e) => {
                    instance.destroy_instance(None);
                    return Err(e);
                }
            };
            let mut renderer = VkRenderer {
                swap_fn: ash::khr::swapchain::Device::new(&instance, &device),
                push_fn: ash::khr::push_descriptor::Device::new(&instance, &device),
                queue: device.get_device_queue(family, 0),
                memory: instance.get_physical_device_memory_properties(physical),
                _entry: entry,
                instance,
                surface_fn,
                surface,
                physical,
                device,
                gpu,
                pool: vk::CommandPool::null(),
                frame_cmd: vk::CommandBuffer::null(),
                upload_cmd: vk::CommandBuffer::null(),
                uploading_cmd: false,
                fence: vk::Fence::null(),
                submitted: false,
                acquire: vk::Semaphore::null(),
                swapchain: vk::SwapchainKHR::null(),
                format: RGBA8,
                extent: vk::Extent2D::default(),
                images: Vec::new(),
                acquired: None,
                frame_number: 0,
                stale: false,
                set_layout: vk::DescriptorSetLayout::null(),
                layout: vk::PipelineLayout::null(),
                pipelines: [vk::Pipeline::null(); PIPELINES],
                linear: vk::Sampler::null(),
                nearest_repeat: vk::Sampler::null(),
                staging: Buf::NULL,
                staging_used: 0,
                rect_vertices: Buf::NULL,
                tex_vertices: Buf::NULL,
                indices: Buf::NULL,
                garbage: Vec::new(),
                blank: None,
                atlas: None,
                image: None,
                wallpaper: None,
                wallpaper_id: None,
                wallpaper_blur: None,
                wallpaper_cover: (0, 0),
                wallpaper_thumbs: None,
                bayer: None,
                render_size: (size.0.max(1), size.1.max(1)),
                async_wallpaper: false,
                pending_wallpaper: None,
                uploading: None,
                wallpaper_failed: None,
                content_changed: true,
                cache_file: cache_dir.map(|dir| dir.join("vulkan-pipelines.bin")),
            };
            // From here a failure drops the renderer, which destroys whatever was made.
            renderer.create_resources(family)?;
            log::info!(
                "Vulkan renderer on {} ({}), {} swapchain images of {}x{}, ready in {} ms",
                renderer.gpu.0,
                renderer.gpu.1,
                renderer.images.len(),
                renderer.extent.width,
                renderer.extent.height,
                started.elapsed().as_millis()
            );
            Ok(renderer)
        }
    }

    /// The surface of `window` and a Vulkan 1.3 device that presents on it.
    #[allow(clippy::type_complexity)]
    unsafe fn open_device(
        entry: &ash::Entry,
        instance: &ash::Instance,
        surface_fn: &ash::khr::surface::Instance,
        window: *mut c_void,
    ) -> Result<
        (
            vk::SurfaceKHR,
            vk::PhysicalDevice,
            u32,
            ash::Device,
            (String, String),
        ),
        String,
    > {
        let physical = *instance
            .enumerate_physical_devices()
            .or_else(err("vkEnumeratePhysicalDevices"))?
            .first()
            .ok_or("no Vulkan device")?;
        let properties = instance.get_physical_device_properties(physical);
        if properties.api_version < vk::API_VERSION_1_3 {
            return Err("the GPU's Vulkan is older than 1.3".into());
        }
        let wanted = [
            ash::khr::swapchain::NAME,
            ash::khr::push_descriptor::NAME,
            ash::khr::incremental_present::NAME,
        ];
        let available = instance
            .enumerate_device_extension_properties(physical)
            .or_else(err("vkEnumerateDeviceExtensionProperties"))?;
        for name in wanted {
            if !available
                .iter()
                .any(|e| e.extension_name_as_c_str().ok() == Some(name))
            {
                return Err(format!("the GPU has no {}", name.to_string_lossy()));
            }
        }
        let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut features13);
        instance.get_physical_device_features2(physical, &mut features);
        if features13.dynamic_rendering == vk::FALSE {
            return Err("the GPU has no dynamic rendering".into());
        }
        let surface = ash::khr::android_surface::Instance::new(entry, instance)
            .create_android_surface(
                &vk::AndroidSurfaceCreateInfoKHR::default().window(window.cast()),
                None,
            )
            .or_else(err("vkCreateAndroidSurfaceKHR"))?;
        let family = instance
            .get_physical_device_queue_family_properties(physical)
            .iter()
            .enumerate()
            .position(|(index, family)| {
                family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_fn
                        .get_physical_device_surface_support(physical, index as u32, surface)
                        .unwrap_or(false)
            });
        let Some(family) = family else {
            surface_fn.destroy_surface(surface, None);
            return Err("no queue draws on the window".into());
        };
        let priorities = [1.0];
        let queues = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(family as u32)
            .queue_priorities(&priorities)];
        let names = wanted.map(|name| name.as_ptr());
        let mut enabled = vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);
        let device = instance.create_device(
            physical,
            &vk::DeviceCreateInfo::default()
                .queue_create_infos(&queues)
                .enabled_extension_names(&names)
                .push_next(&mut enabled),
            None,
        );
        let device = match device {
            Ok(device) => device,
            Err(e) => {
                surface_fn.destroy_surface(surface, None);
                return Err(format!("vkCreateDevice: {e}"));
            }
        };
        let name = properties
            .device_name_as_c_str()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let v = properties.api_version;
        let version = format!(
            "Vulkan {}.{}.{}",
            vk::api_version_major(v),
            vk::api_version_minor(v),
            vk::api_version_patch(v)
        );
        Ok((surface, physical, family as u32, device, (name, version)))
    }

    unsafe fn create_resources(&mut self, family: u32) -> Result<(), String> {
        let device = &self.device;
        self.pool = device
            .create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
            .or_else(err("vkCreateCommandPool"))?;
        let buffers = device
            .allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.pool)
                    .command_buffer_count(2),
            )
            .or_else(err("vkAllocateCommandBuffers"))?;
        (self.frame_cmd, self.upload_cmd) = (buffers[0], buffers[1]);
        self.fence = device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .or_else(err("vkCreateFence"))?;
        self.acquire = device
            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            .or_else(err("vkCreateSemaphore"))?;
        let sampler = |filter, mode| {
            device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(filter)
                        .min_filter(filter)
                        .address_mode_u(mode)
                        .address_mode_v(mode)
                        .address_mode_w(mode),
                    None,
                )
                .or_else(err("vkCreateSampler"))
        };
        self.linear = sampler(vk::Filter::LINEAR, vk::SamplerAddressMode::CLAMP_TO_EDGE)?;
        self.nearest_repeat = sampler(vk::Filter::NEAREST, vk::SamplerAddressMode::REPEAT)?;

        self.create_swapchain()?;
        self.create_pipelines()?;

        self.staging = self.buffer(STAGING_BYTES, vk::BufferUsageFlags::TRANSFER_SRC)?;
        self.rect_vertices = self.buffer(64 << 10, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        self.tex_vertices = self.buffer(256 << 10, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let mut indices = Vec::with_capacity(MAX_QUADS_PER_BATCH as usize * 6);
        for quad in 0..MAX_QUADS_PER_BATCH {
            indices.extend(QUAD_INDICES.iter().map(|i| (quad * 4) as u16 + i));
        }
        let bytes: &[u8] = bytemuck::cast_slice(&indices);
        self.indices = self.buffer(bytes.len() as u64, vk::BufferUsageFlags::INDEX_BUFFER)?;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.indices.ptr, bytes.len());

        let blank = self.texture(1, 1, RGBA8, false)?;
        self.upload(blank, [0, 0, 1, 1], 4, |to| to.fill(0));
        self.blank = Some(blank);
        Ok(())
    }

    unsafe fn create_swapchain(&mut self) -> Result<(), String> {
        let caps = self
            .surface_fn
            .get_physical_device_surface_capabilities(self.physical, self.surface)
            .or_else(err("surface capabilities"))?;
        let formats = self
            .surface_fn
            .get_physical_device_surface_formats(self.physical, self.surface)
            .or_else(err("surface formats"))?;
        let format = formats
            .iter()
            .find(|f| f.format == RGBA8 && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .or(formats.first())
            .ok_or("the surface has no format")?;
        // One more than the least, as OpenGL's three buffers: with two, a frame that is a little
        // late waits a whole refresh for the image on screen to come back.
        let mut count = caps.min_image_count + 1;
        if caps.max_image_count > 0 {
            count = count.min(caps.max_image_count);
        }
        let extent = vk::Extent2D {
            width: self.render_size.0.clamp(
                caps.min_image_extent.width,
                caps.max_image_extent.width.max(1),
            ),
            height: self.render_size.1.clamp(
                caps.min_image_extent.height,
                caps.max_image_extent.height.max(1),
            ),
        };
        // Nothing is behind the launcher: an opaque layer is the cheapest for the compositor.
        let alpha = [
            vk::CompositeAlphaFlagsKHR::OPAQUE,
            vk::CompositeAlphaFlagsKHR::INHERIT,
        ]
        .into_iter()
        .find(|a| caps.supported_composite_alpha.contains(*a))
        .unwrap_or(vk::CompositeAlphaFlagsKHR::OPAQUE);
        let old = self.swapchain;
        let swapchain = self
            .swap_fn
            .create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(self.surface)
                    .min_image_count(count)
                    .image_format(format.format)
                    .image_color_space(format.color_space)
                    .image_extent(extent)
                    .image_array_layers(1)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .pre_transform(caps.current_transform)
                    .composite_alpha(alpha)
                    .present_mode(vk::PresentModeKHR::FIFO)
                    .clipped(true)
                    .old_swapchain(old),
                None,
            )
            .or_else(err("vkCreateSwapchainKHR"))?;
        self.destroy_swapchain_images();
        self.swap_fn.destroy_swapchain(old, None);
        self.swapchain = swapchain;
        self.format = format.format;
        self.extent = extent;
        for image in self
            .swap_fn
            .get_swapchain_images(swapchain)
            .or_else(err("vkGetSwapchainImagesKHR"))?
        {
            let view = self.view(image, format.format)?;
            let drawn = self
                .device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .or_else(err("vkCreateSemaphore"))?;
            self.images.push(SwapImage {
                image,
                view,
                drawn,
                presented: None,
            });
        }
        self.acquired = None;
        self.stale = false;
        self.content_changed = true;
        Ok(())
    }

    unsafe fn destroy_swapchain_images(&mut self) {
        for image in self.images.drain(..) {
            self.device.destroy_image_view(image.view, None);
            self.device.destroy_semaphore(image.drawn, None);
        }
    }

    unsafe fn create_pipelines(&mut self) -> Result<(), String> {
        let device = &self.device;
        let sampled = |binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        };
        self.set_layout = device
            .create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default()
                    .flags(vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR)
                    .bindings(&[sampled(0), sampled(1)]),
                None,
            )
            .or_else(err("vkCreateDescriptorSetLayout"))?;
        self.layout = device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[self.set_layout])
                    .push_constant_ranges(&[vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                        .size(32)]),
                None,
            )
            .or_else(err("vkCreatePipelineLayout"))?;

        // The driver's compiled pipelines, kept between starts so they are not compiled again.
        let saved = self
            .cache_file
            .as_ref()
            .and_then(|file| std::fs::read(file).ok())
            .unwrap_or_default();
        let cache = device
            .create_pipeline_cache(
                &vk::PipelineCacheCreateInfo::default().initial_data(&saved),
                None,
            )
            .or_else(|_| {
                device.create_pipeline_cache(&vk::PipelineCacheCreateInfo::default(), None)
            })
            .or_else(err("vkCreatePipelineCache"))?;

        macro_rules! spv {
            ($name:literal) => {
                include_bytes!(concat!("../../shaders/", $name, ".spv")).as_slice()
            };
        }
        let rect_stride = size_of::<RectVertex>() as u32;
        let tex_stride = size_of::<TexVertex>() as u32;
        // (vertex shader, fragment shader, vertex stride, vec4 attributes, blended, format)
        #[allow(clippy::type_complexity)]
        let specs: [(&[u8], &[u8], u32, u32, bool, vk::Format); PIPELINES] = [
            (
                spv!("rect.vert"),
                spv!("rect.frag"),
                rect_stride,
                7,
                true,
                self.format,
            ),
            (
                spv!("tex.vert"),
                spv!("text.frag"),
                tex_stride,
                5,
                true,
                self.format,
            ),
            (
                spv!("tex.vert"),
                spv!("image.frag"),
                tex_stride,
                5,
                true,
                self.format,
            ),
            (
                spv!("tex.vert"),
                spv!("glass.frag"),
                tex_stride,
                5,
                true,
                self.format,
            ),
            (
                spv!("wallpaper.vert"),
                spv!("wallpaper.frag"),
                tex_stride,
                2,
                true,
                self.format,
            ),
            (spv!("reduce.vert"), spv!("reduce.frag"), 0, 0, false, RGBA8),
        ];
        let mut result = Ok(());
        for (index, (vertex, fragment, stride, attributes, blended, format)) in
            specs.into_iter().enumerate()
        {
            match self.pipeline(cache, vertex, fragment, stride, attributes, blended, format) {
                Ok(pipeline) => self.pipelines[index] = pipeline,
                Err(e) => result = Err(e),
            }
        }
        if result.is_ok() {
            if let (Some(file), Ok(data)) = (
                self.cache_file.as_ref(),
                self.device.get_pipeline_cache_data(cache),
            ) {
                if data.len() != saved.len() {
                    let _ = std::fs::write(file, data);
                }
            }
        }
        self.device.destroy_pipeline_cache(cache, None);
        result
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn pipeline(
        &self,
        cache: vk::PipelineCache,
        vertex: &[u8],
        fragment: &[u8],
        stride: u32,
        attributes: u32,
        blended: bool,
        format: vk::Format,
    ) -> Result<vk::Pipeline, String> {
        let device = &self.device;
        let module = |spv: &[u8]| {
            let words = ash::util::read_spv(&mut std::io::Cursor::new(spv))
                .map_err(|e| format!("bad SPIR-V: {e}"))?;
            device
                .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None)
                .or_else(err("vkCreateShaderModule"))
        };
        let vertex = module(vertex)?;
        let fragment = match module(fragment) {
            Ok(fragment) => fragment,
            Err(e) => {
                device.destroy_shader_module(vertex, None);
                return Err(e);
            }
        };
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment)
                .name(c"main"),
        ];
        let bindings = [vk::VertexInputBindingDescription::default().stride(stride)];
        let attributes: Vec<_> = (0..attributes)
            .map(|index| {
                vk::VertexInputAttributeDescription::default()
                    .location(index)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(index * 16)
            })
            .collect();
        let mut input = vk::PipelineVertexInputStateCreateInfo::default();
        if !attributes.is_empty() {
            input = input
                .vertex_binding_descriptions(&bindings)
                .vertex_attribute_descriptions(&attributes);
        }
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let raster = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .line_width(1.0);
        let samples = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        // Premultiplied alpha, as the OpenGL renderer blends.
        let attachments = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(blended)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments);
        let dynamic = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic);
        let formats = [format];
        let mut rendering =
            vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&formats);
        let info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&input)
            .input_assembly_state(&assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&samples)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(self.layout)
            .push_next(&mut rendering);
        let made = device.create_graphics_pipelines(cache, &[info], None);
        device.destroy_shader_module(vertex, None);
        device.destroy_shader_module(fragment, None);
        match made {
            Ok(pipelines) => Ok(pipelines[0]),
            Err((_, e)) => Err(format!("vkCreateGraphicsPipelines: {e}")),
        }
    }

    fn memory_type(&self, bits: u32, flags: vk::MemoryPropertyFlags) -> Result<u32, String> {
        (0..self.memory.memory_type_count)
            .find(|i| {
                bits & (1 << i) != 0
                    && self.memory.memory_types[*i as usize]
                        .property_flags
                        .contains(flags)
            })
            .ok_or_else(|| "no memory type fits".to_string())
    }

    /// A buffer in memory the CPU sees, mapped for its whole life.
    unsafe fn buffer(&self, size: u64, usage: vk::BufferUsageFlags) -> Result<Buf, String> {
        let device = &self.device;
        let buffer = device
            .create_buffer(
                &vk::BufferCreateInfo::default().size(size).usage(usage),
                None,
            )
            .or_else(err("vkCreateBuffer"))?;
        let needs = device.get_buffer_memory_requirements(buffer);
        let made = self
            .memory_type(
                needs.memory_type_bits,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .and_then(|kind| {
                device
                    .allocate_memory(
                        &vk::MemoryAllocateInfo::default()
                            .allocation_size(needs.size)
                            .memory_type_index(kind),
                        None,
                    )
                    .or_else(err("vkAllocateMemory"))
            })
            .and_then(|memory| {
                let mapped = device.bind_buffer_memory(buffer, memory, 0).and_then(|()| {
                    device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())
                });
                match mapped {
                    Ok(ptr) => Ok((memory, ptr.cast::<u8>())),
                    Err(e) => {
                        device.free_memory(memory, None);
                        Err(format!("vkMapMemory: {e}"))
                    }
                }
            });
        match made {
            Ok((memory, ptr)) => Ok(Buf {
                buffer,
                memory,
                ptr,
                size,
            }),
            Err(e) => {
                device.destroy_buffer(buffer, None);
                Err(e)
            }
        }
    }

    unsafe fn destroy_buffer(&self, buf: &Buf) {
        self.device.destroy_buffer(buf.buffer, None);
        self.device.free_memory(buf.memory, None);
    }

    unsafe fn view(&self, image: vk::Image, format: vk::Format) -> Result<vk::ImageView, String> {
        self.device
            .create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(COLOR_RANGE),
                None,
            )
            .or_else(err("vkCreateImageView"))
    }

    /// A texture programs can sample at once (its content is undefined until uploaded or drawn).
    /// `target`: it can also be drawn into.
    unsafe fn texture(
        &mut self,
        width: u32,
        height: u32,
        format: vk::Format,
        target: bool,
    ) -> Result<Tex, String> {
        let mut usage = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST;
        if target {
            usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        let device = &self.device;
        let image = device
            .create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(format)
                    .extent(vk::Extent3D {
                        width,
                        height,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .usage(usage),
                None,
            )
            .or_else(err("vkCreateImage"))?;
        let needs = device.get_image_memory_requirements(image);
        let memory = self
            .memory_type(
                needs.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .or_else(|_| self.memory_type(needs.memory_type_bits, vk::MemoryPropertyFlags::empty()))
            .and_then(|kind| {
                device
                    .allocate_memory(
                        &vk::MemoryAllocateInfo::default()
                            .allocation_size(needs.size)
                            .memory_type_index(kind),
                        None,
                    )
                    .or_else(err("vkAllocateMemory"))
            });
        let memory = match memory {
            Ok(memory) => memory,
            Err(e) => {
                device.destroy_image(image, None);
                return Err(e);
            }
        };
        let view = device
            .bind_image_memory(image, memory, 0)
            .or_else(err("vkBindImageMemory"))
            .and_then(|()| self.view(image, format));
        let view = match view {
            Ok(view) => view,
            Err(e) => {
                device.destroy_image(image, None);
                device.free_memory(memory, None);
                return Err(e);
            }
        };
        let cmd = self.uploads();
        self.barrier(cmd, image, vk::ImageLayout::UNDEFINED, READ);
        Ok(Tex {
            image,
            memory,
            view,
            width,
            height,
        })
    }

    unsafe fn destroy_texture(&self, tex: Tex) {
        self.device.destroy_image_view(tex.view, None);
        self.device.destroy_image(tex.image, None);
        self.device.free_memory(tex.memory, None);
    }

    unsafe fn barrier(
        &self,
        cmd: vk::CommandBuffer,
        image: vk::Image,
        from: vk::ImageLayout,
        to: vk::ImageLayout,
    ) {
        let access = |layout| match layout {
            vk::ImageLayout::TRANSFER_DST_OPTIMAL => vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => {
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            }
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => vk::AccessFlags::SHADER_READ,
            _ => vk::AccessFlags::empty(),
        };
        self.device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[vk::ImageMemoryBarrier::default()
                .src_access_mask(access(from))
                .dst_access_mask(access(to))
                .old_layout(from)
                .new_layout(to)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(COLOR_RANGE)],
        );
    }

    /// Waits for the pending submit, after which what it used can be written again or freed.
    unsafe fn settle(&mut self) {
        if self.submitted {
            let _ = self.device.wait_for_fences(&[self.fence], true, u64::MAX);
            let _ = self.device.reset_fences(&[self.fence]);
            self.submitted = false;
        }
        for tex in std::mem::take(&mut self.garbage) {
            self.destroy_texture(tex);
        }
    }

    /// The command buffer of the texture uploads since the last submit, begun if it was not.
    unsafe fn uploads(&mut self) -> vk::CommandBuffer {
        if !self.uploading_cmd {
            self.settle();
            self.staging_used = 0;
            let _ = self.device.begin_command_buffer(
                self.upload_cmd,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            );
            self.uploading_cmd = true;
        }
        self.upload_cmd
    }

    /// Submits the uploads recorded so far and waits for them: the staging buffer is full.
    unsafe fn flush_uploads(&mut self) {
        if !self.uploading_cmd {
            return;
        }
        let _ = self.device.end_command_buffer(self.upload_cmd);
        self.uploading_cmd = false;
        let submitted = self.device.queue_submit(
            self.queue,
            &[vk::SubmitInfo::default().command_buffers(&[self.upload_cmd])],
            self.fence,
        );
        self.submitted = submitted.is_ok();
        self.settle();
    }

    /// Replaces a region (`[x, y, width, height]`) of `tex` with the `bytes_per_pixel * width *
    /// height` bytes `fill` writes.
    unsafe fn upload(
        &mut self,
        tex: Tex,
        [x, y, w, h]: [u32; 4],
        bytes_per_pixel: u32,
        fill: impl FnOnce(&mut [u8]),
    ) {
        let size = u64::from(w) * u64::from(h) * u64::from(bytes_per_pixel);
        assert!(
            size <= STAGING_BYTES,
            "the upload is larger than the staging buffer"
        );
        let mut cmd = self.uploads();
        // A multiple of four bytes, which every format here is aligned to.
        let mut offset = self.staging_used.next_multiple_of(4);
        if offset + size > STAGING_BYTES {
            self.flush_uploads();
            cmd = self.uploads();
            offset = 0;
        }
        fill(std::slice::from_raw_parts_mut(
            self.staging.ptr.add(offset as usize),
            size as usize,
        ));
        self.staging_used = offset + size;
        self.barrier(cmd, tex.image, READ, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        self.device.cmd_copy_buffer_to_image(
            cmd,
            self.staging.buffer,
            tex.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[vk::BufferImageCopy::default()
                .buffer_offset(offset)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D {
                    x: x as i32,
                    y: y as i32,
                    z: 0,
                })
                .image_extent(vk::Extent3D {
                    width: w,
                    height: h,
                    depth: 1,
                })],
        );
        self.barrier(cmd, tex.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, READ);
    }

    unsafe fn push_textures(&self, cmd: vk::CommandBuffer, texture: vk::ImageView) {
        let bayer = self.bayer.or(self.blank).map_or(texture, |t| t.view);
        let first = [vk::DescriptorImageInfo {
            sampler: self.linear,
            image_view: texture,
            image_layout: READ,
        }];
        let second = [vk::DescriptorImageInfo {
            sampler: self.nearest_repeat,
            image_view: bayer,
            image_layout: READ,
        }];
        let write = |binding, info| {
            vk::WriteDescriptorSet::default()
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(info)
        };
        self.push_fn.cmd_push_descriptor_set(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            self.layout,
            0,
            &[write(0, &first), write(1, &second)],
        );
    }

    unsafe fn set_area(&self, cmd: vk::CommandBuffer, width: u32, height: u32, area: vk::Rect2D) {
        self.device.cmd_set_viewport(
            cmd,
            0,
            &[vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }],
        );
        self.device.cmd_set_scissor(cmd, 0, &[area]);
    }

    /// Draws `source` into `destination` with the reduction shader (`spread` above one blurs).
    unsafe fn reduce(&mut self, source: Tex, destination: Tex, spread: f32) {
        let cmd = self.uploads();
        let (width, height) = (destination.width, destination.height);
        let area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D { width, height },
        };
        let target = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
        self.barrier(cmd, destination.image, READ, target);
        let attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(destination.view)
            .image_layout(target)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)];
        self.device.cmd_begin_rendering(
            cmd,
            &vk::RenderingInfo::default()
                .render_area(area)
                .layer_count(1)
                .color_attachments(&attachments),
        );
        self.set_area(cmd, width, height, area);
        self.device
            .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipelines[REDUCE]);
        self.push_textures(cmd, source.view);
        let constants = [
            0.25 * spread / width as f32,
            0.25 * spread / height as f32,
            width as f32,
            height as f32,
            0.0,
            0.0,
            0.0,
            0.0,
        ];
        self.device.cmd_push_constants(
            cmd,
            self.layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::cast_slice(&constants),
        );
        self.device.cmd_draw(cmd, 3, 1, 0, 0);
        self.device.cmd_end_rendering(cmd);
        self.barrier(cmd, destination.image, target, READ);
    }

    /// Creates (or replaces) the 8-bit glyph atlas.
    fn set_atlas(&mut self, width: u32, height: u32, pixels: &[u8]) {
        self.content_changed = true;
        assert_eq!(pixels.len(), (width * height) as usize);
        // SAFETY: the device is alive; the texture and the upload follow the renderer's rules.
        unsafe {
            let atlas = match self.atlas {
                Some(atlas) if (atlas.width, atlas.height) == (width, height) => atlas,
                old => {
                    self.garbage.extend(old);
                    self.atlas = None;
                    let Ok(atlas) = self.texture(width, height, vk::Format::R8_UNORM, false) else {
                        log::error!("cannot create the atlas texture");
                        return;
                    };
                    self.atlas = Some(atlas);
                    atlas
                }
            };
            self.upload(atlas, [0, 0, width, height], 1, |to| {
                to.copy_from_slice(pixels)
            });
        }
    }

    /// Uploads whatever changed in the text engine's atlas since the last call.
    pub fn sync_text_atlas(&mut self, engine: &mut TextEngine) {
        let updates = engine.take_updates();
        if updates.iter().any(|u| matches!(u, Update::Full { .. })) {
            let atlas = engine.atlas();
            self.set_atlas(atlas.width(), atlas.height(), atlas.pixels());
            return;
        }
        for update in updates {
            if let (Update::Region(slot), Some(atlas)) = (update, self.atlas) {
                let pixels = engine.atlas().region_pixels(slot);
                // Not a change of content: see `Gles2Backend::update_atlas`.
                // SAFETY: the region is inside the atlas the engine and the texture share.
                unsafe {
                    self.upload(atlas, [slot.x, slot.y, slot.width, slot.height], 1, |to| {
                        to.copy_from_slice(&pixels)
                    });
                }
            }
        }
    }

    /// Creates the RGBA icon texture empty, if it does not exist yet at this size.
    pub fn ensure_image(&mut self, width: u32, height: u32) {
        if self
            .image
            .is_some_and(|image| (image.width, image.height) == (width, height))
        {
            return;
        }
        self.garbage.extend(self.image.take());
        // SAFETY: see `set_atlas`.
        match unsafe { self.texture(width, height, RGBA8, false) } {
            Ok(image) => self.image = Some(image),
            Err(e) => log::error!("cannot create the image texture: {e}"),
        }
    }

    /// Replaces a region of the icon texture with premultiplied RGBA pixels.
    pub fn update_image(&mut self, region: [u32; 4], rgba: &[u8]) {
        self.content_changed = true;
        let [x, y, w, h] = region;
        assert_eq!(rgba.len(), (w * h * 4) as usize);
        let Some(image) = self.image else {
            return;
        };
        assert!(x + w <= image.width && y + h <= image.height);
        // SAFETY: the region is inside the texture.
        unsafe { self.upload(image, region, 4, |to| to.copy_from_slice(rgba)) };
    }

    pub fn set_async_wallpaper(&mut self, enabled: bool) {
        self.async_wallpaper = enabled;
    }

    pub fn poll_wallpaper(&mut self, id: WallpaperId) {
        self.prepare_wallpaper(id, !self.async_wallpaper);
    }

    pub fn wallpaper_uploading(&self) -> bool {
        self.uploading.is_some()
    }

    pub fn has_wallpaper(&self) -> bool {
        self.wallpaper.is_some()
    }

    pub fn gpu_strings(&self) -> (String, String) {
        self.gpu.clone()
    }

    pub fn set_render_size(&mut self, width: u32, height: u32) {
        let size = (width.max(1), height.max(1));
        if self.render_size != size {
            self.render_size = size;
            self.content_changed = true;
            self.stale = true;
        }
    }

    pub fn take_content_changed(&mut self) -> bool {
        std::mem::take(&mut self.content_changed)
    }

    /// Puts on the GPU the textures `frame` samples.
    pub fn prepare(&mut self, frame: &Frame) {
        if frame.batches.is_empty() {
            return;
        }
        if frame.uses_wallpaper() {
            let id = frame.wallpaper_id.unwrap_or_default();
            // Waiting is only acceptable when there is nothing to show yet.
            let block = !self.async_wallpaper || self.wallpaper.is_none();
            self.prepare_wallpaper(id, block);
        }
        if self.wallpaper_thumbs.is_none()
            && frame
                .batches
                .iter()
                .any(|b| b.texture == ImageTexture::WallpaperThumbs)
        {
            self.content_changed = true;
            let rgba = wallpaper::decompress_thumbnails();
            let (width, height) = (wallpaper::THUMBNAILS_WIDTH, wallpaper::THUMBNAILS_HEIGHT);
            // SAFETY: the thumbnails are exactly the texture's size.
            unsafe {
                if let Ok(thumbs) = self.texture(width, height, RGBA8, false) {
                    self.upload(thumbs, [0, 0, width, height], 4, |to| {
                        to.copy_from_slice(&rgba)
                    });
                    self.wallpaper_thumbs = Some(thumbs);
                }
            }
        }
    }

    /// The same steps as `Gles2Backend::prepare_wallpaper`: decode (on the worker thread when
    /// asynchronous), then upload, a slice per frame when it replaces another wallpaper.
    fn prepare_wallpaper(&mut self, id: WallpaperId, block: bool) {
        let cover = cover_size(self.render_size);
        if let Some(upload) = &self.uploading {
            if (upload.id, upload.cover) == (id, cover) {
                self.step_wallpaper_upload(block);
                return;
            }
            if let Some(upload) = self.uploading.take() {
                self.garbage.push(upload.source);
            }
        }
        if self.wallpaper.is_some()
            && self.wallpaper_id == Some(id)
            && self.wallpaper_cover == cover
        {
            return;
        }
        self.content_changed = true;
        let image = id.wallpaper();
        if self.wallpaper_failed == Some(id) {
            return;
        }
        let started = std::time::Instant::now();
        let loaded_rgb = if self.async_wallpaper {
            if self.pending_wallpaper.as_ref().map(|(pending, _)| *pending) != Some(id) {
                self.pending_wallpaper = Some((id, wallpaper::load_in_background(id)));
            }
            let Some((_, receiver)) = self.pending_wallpaper.as_ref() else {
                return;
            };
            let result = if block {
                receiver.recv().map_err(|_| ())
            } else {
                match receiver.try_recv() {
                    Ok(rgb) => Ok(rgb),
                    Err(std::sync::mpsc::TryRecvError::Empty) => return,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(()),
                }
            };
            self.pending_wallpaper = None;
            result.unwrap_or(None)
        } else {
            image.load_rgb()
        };
        // SAFETY: the textures made here are the renderer's own.
        let source = loaded_rgb.and_then(|rgb| unsafe {
            if self.bayer.is_none() {
                let bayer = self.texture(4, 4, vk::Format::R8_UNORM, false).ok()?;
                self.upload(bayer, [0, 0, 4, 4], 1, |to| to.copy_from_slice(&BAYER_4X4));
                self.bayer = Some(bayer);
            }
            let source = self.texture(image.width, image.height, RGBA8, false).ok()?;
            Some((rgb, source))
        });
        let Some((rgb, source)) = source else {
            log::error!("cannot load the wallpaper {}", id.as_str());
            self.wallpaper_failed = Some(id);
            return;
        };
        self.uploading = Some(WallpaperUpload {
            id,
            cover,
            rgb,
            rows_done: 0,
            source,
            started,
            loaded: started.elapsed(),
        });
        self.step_wallpaper_upload(!(self.async_wallpaper && self.wallpaper.is_some()));
    }

    fn step_wallpaper_upload(&mut self, all: bool) {
        let Some(mut upload) = self.uploading.take() else {
            return;
        };
        self.content_changed = true;
        let image = upload.id.wallpaper();
        let rows = if all {
            image.height
        } else {
            WALLPAPER_UPLOAD_ROWS
        }
        .min(image.height - upload.rows_done);
        let row_bytes = (image.width * 3) as usize;
        let from = upload.rows_done as usize * row_bytes;
        let rgb = &upload.rgb[from..from + rows as usize * row_bytes];
        // SAFETY: the rows are inside the source texture.
        unsafe {
            self.upload(
                upload.source,
                [0, upload.rows_done, image.width, rows],
                4,
                |to| {
                    // Vulkan has no 24-bit texture on this GPU: the alpha is added here.
                    for (pixel, rgb) in to
                        .as_chunks_mut::<4>()
                        .0
                        .iter_mut()
                        .zip(rgb.as_chunks::<3>().0)
                    {
                        *pixel = [rgb[0], rgb[1], rgb[2], 255];
                    }
                },
            );
        }
        upload.rows_done += rows;
        if upload.rows_done < image.height {
            self.uploading = Some(upload);
            return;
        }
        let WallpaperUpload {
            id,
            cover,
            source,
            started,
            loaded,
            ..
        } = upload;
        let mut texture = source;
        // SAFETY: the textures drawn into are made as targets.
        unsafe {
            if cover.0 < image.width && cover.1 < image.height {
                if let Ok(reduced) = self.texture(cover.0, cover.1, RGBA8, true) {
                    self.reduce(source, reduced, 1.0);
                    self.garbage.push(source);
                    texture = reduced;
                }
            }
            self.garbage.extend(self.wallpaper.replace(texture));
            self.wallpaper_id = Some(id);
            self.wallpaper_cover = cover;
            self.blur_wallpaper(texture);
        }
        log::info!(
            "wallpaper {} ready: load {} ms, total {} ms ({}x{})",
            id.as_str(),
            loaded.as_millis(),
            started.elapsed().as_millis(),
            texture.width,
            texture.height
        );
    }

    /// The small blurred copy glass shows: reduced by eight, then four box passes at that size.
    unsafe fn blur_wallpaper(&mut self, wallpaper: Tex) {
        self.garbage.extend(self.wallpaper_blur.take());
        let quarter = (wallpaper.width.div_ceil(4), wallpaper.height.div_ceil(4));
        let small = (quarter.0.div_ceil(2), quarter.1.div_ceil(2));
        let (Ok(first), Ok(mut front), Ok(mut back)) = (
            self.texture(quarter.0, quarter.1, RGBA8, true),
            self.texture(small.0, small.1, RGBA8, true),
            self.texture(small.0, small.1, RGBA8, true),
        ) else {
            // Leaked until the window goes: this only fails without memory.
            return;
        };
        self.reduce(wallpaper, first, 1.0);
        self.reduce(first, front, 1.0);
        for _ in 0..4 {
            self.reduce(front, back, 6.0);
            std::mem::swap(&mut front, &mut back);
        }
        self.wallpaper_blur = Some(front);
        self.garbage.extend([first, back]);
    }

    unsafe fn acquire_image(&mut self) -> Result<u32, String> {
        if let Some(index) = self.acquired {
            return Ok(index);
        }
        // The semaphore of the last acquire must have been waited on by its frame.
        if !self.uploading_cmd {
            self.settle();
        }
        for _ in 0..2 {
            if self.stale {
                let _ = self.device.device_wait_idle();
                self.create_swapchain()?;
            }
            match self.swap_fn.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.acquire,
                vk::Fence::null(),
            ) {
                Ok((index, _suboptimal)) => {
                    self.acquired = Some(index);
                    return Ok(index);
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.stale = true,
                Err(e) => return Err(format!("vkAcquireNextImageKHR: {e}")),
            }
        }
        Err("the swapchain stays out of date".into())
    }

    /// How many frames ago the image of the next frame was on screen, or 0 when its content is
    /// unknown and the frame must be drawn whole. What `EGL_EXT_buffer_age` is to OpenGL.
    pub fn buffer_age(&mut self) -> usize {
        // SAFETY: the swapchain is alive.
        match unsafe { self.acquire_image() } {
            Ok(index) => self.images[index as usize]
                .presented
                .map_or(0, |at| (self.frame_number - at) as usize),
            Err(_) => 0,
        }
    }

    /// Draws `frame` and shows it. `damage` is an OpenGL scissor box (origin bottom-left): only
    /// that part is drawn, over what the image already had; `None` draws everything.
    pub fn present(
        &mut self,
        viewport: &Viewport,
        clear: Color,
        frame: &Frame,
        damage: Option<[i32; 4]>,
    ) -> Result<(), String> {
        // SAFETY: one frame at a time; everything recorded lives until the next `settle`.
        unsafe {
            let index = self.acquire_image()?;
            if self.uploading_cmd {
                let _ = self.device.end_command_buffer(self.upload_cmd);
            } else {
                self.settle();
            }
            let (width, height) = (self.extent.width, self.extent.height);
            let full = vk::Rect2D {
                offset: vk::Offset2D::default(),
                extent: self.extent,
            };
            let known = self.images[index as usize].presented.is_some();
            let area = match damage {
                Some(scissor) if known => to_rect(scissor, width, height),
                _ => full,
            };
            let whole = area == full;

            self.write_vertices(frame)?;
            let cmd = self.frame_cmd;
            self.device
                .begin_command_buffer(
                    cmd,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .or_else(err("vkBeginCommandBuffer"))?;
            let image = self.images[index as usize].image;
            let target = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
            let before = if known && !whole {
                vk::ImageLayout::PRESENT_SRC_KHR
            } else {
                vk::ImageLayout::UNDEFINED
            };
            self.barrier(cmd, image, before, target);
            let [r, g, b, a] = clear.to_f32();
            let clear = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [r * a, g * a, b * a, a],
                },
            };
            let attachments = [vk::RenderingAttachmentInfo::default()
                .image_view(self.images[index as usize].view)
                .image_layout(target)
                .load_op(if whole {
                    vk::AttachmentLoadOp::CLEAR
                } else {
                    vk::AttachmentLoadOp::LOAD
                })
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(clear)];
            self.device.cmd_begin_rendering(
                cmd,
                &vk::RenderingInfo::default()
                    .render_area(area)
                    .layer_count(1)
                    .color_attachments(&attachments),
            );
            if !whole {
                self.device.cmd_clear_attachments(
                    cmd,
                    &[vk::ClearAttachment {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        color_attachment: 0,
                        clear_value: clear,
                    }],
                    &[vk::ClearRect {
                        rect: area,
                        base_array_layer: 0,
                        layer_count: 1,
                    }],
                );
            }
            self.draw_batches(cmd, viewport, frame, area);
            self.device.cmd_end_rendering(cmd);
            self.barrier(cmd, image, target, vk::ImageLayout::PRESENT_SRC_KHR);
            self.device
                .end_command_buffer(cmd)
                .or_else(err("vkEndCommandBuffer"))?;

            let drawn = [self.images[index as usize].drawn];
            let both = [self.upload_cmd, cmd];
            let commands = if self.uploading_cmd {
                &both[..]
            } else {
                &both[1..]
            };
            self.uploading_cmd = false;
            self.acquired = None;
            self.device
                .queue_submit(
                    self.queue,
                    &[vk::SubmitInfo::default()
                        .wait_semaphores(&[self.acquire])
                        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                        .command_buffers(commands)
                        .signal_semaphores(&drawn)],
                    self.fence,
                )
                .or_else(err("vkQueueSubmit"))?;
            self.submitted = true;

            // What changed on screen, for the compositor (`VK_KHR_incremental_present`).
            let rectangles = [vk::RectLayerKHR {
                offset: area.offset,
                extent: area.extent,
                layer: 0,
            }];
            let regions = [vk::PresentRegionKHR::default().rectangles(&rectangles)];
            let mut changed = vk::PresentRegionsKHR::default().regions(&regions);
            let swapchains = [self.swapchain];
            let indices = [index];
            let mut info = vk::PresentInfoKHR::default()
                .wait_semaphores(&drawn)
                .swapchains(&swapchains)
                .image_indices(&indices);
            if !whole {
                info = info.push_next(&mut changed);
            }
            self.images[index as usize].presented = Some(self.frame_number);
            self.frame_number += 1;
            match self.swap_fn.queue_present(self.queue, &info) {
                Ok(_) => Ok(()),
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.stale = true;
                    Ok(())
                }
                Err(e) => Err(format!("vkQueuePresentKHR: {e}")),
            }
        }
    }

    /// Copies the frame's vertices to the GPU: rectangles in one buffer, text then images in
    /// the other, as `Gles2Backend` lays them out.
    unsafe fn write_vertices(&mut self, frame: &Frame) -> Result<(), String> {
        let rects: &[u8] = bytemuck::cast_slice(&frame.rect_vertices);
        let text: &[u8] = bytemuck::cast_slice(&frame.text_vertices);
        let images: &[u8] = bytemuck::cast_slice(&frame.image_vertices);
        if rects.len() as u64 > self.rect_vertices.size {
            let grown = self.buffer(
                (rects.len() as u64).next_power_of_two(),
                vk::BufferUsageFlags::VERTEX_BUFFER,
            )?;
            let old = std::mem::replace(&mut self.rect_vertices, grown);
            self.destroy_buffer(&old);
        }
        if (text.len() + images.len()) as u64 > self.tex_vertices.size {
            let grown = self.buffer(
                ((text.len() + images.len()) as u64).next_power_of_two(),
                vk::BufferUsageFlags::VERTEX_BUFFER,
            )?;
            let old = std::mem::replace(&mut self.tex_vertices, grown);
            self.destroy_buffer(&old);
        }
        std::ptr::copy_nonoverlapping(rects.as_ptr(), self.rect_vertices.ptr, rects.len());
        std::ptr::copy_nonoverlapping(text.as_ptr(), self.tex_vertices.ptr, text.len());
        std::ptr::copy_nonoverlapping(
            images.as_ptr(),
            self.tex_vertices.ptr.add(text.len()),
            images.len(),
        );
        Ok(())
    }

    unsafe fn draw_batches(
        &self,
        cmd: vk::CommandBuffer,
        viewport: &Viewport,
        frame: &Frame,
        area: vk::Rect2D,
    ) {
        let device = &self.device;
        let (width, height) = (self.extent.width, self.extent.height);
        self.set_area(cmd, width, height, area);
        if frame.batches.is_empty() {
            return;
        }
        let constants = [
            viewport.scale,
            viewport.offset_x,
            viewport.offset_y,
            viewport.edge_softness,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            0.0,
        ];
        device.cmd_push_constants(
            cmd,
            self.layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::cast_slice(&constants),
        );
        device.cmd_bind_index_buffer(cmd, self.indices.buffer, 0, vk::IndexType::UINT16);
        let image_quad_base = (frame.text_vertices.len() / 4) as u32;
        let blank = self.blank.map(|t| t.view);
        let (mut pipeline, mut vertices, mut texture) = (usize::MAX, vk::Buffer::null(), None);
        let mut clipped = false;
        for batch in &frame.batches {
            match batch.clip {
                Some(clip) => {
                    let clip = to_rect(Gles2Backend::device_rect(&clip, viewport), width, height);
                    device.cmd_set_scissor(cmd, 0, &[intersect(clip, area)]);
                    clipped = true;
                }
                None if clipped => {
                    device.cmd_set_scissor(cmd, 0, &[area]);
                    clipped = false;
                }
                None => {}
            }
            let blur = self.wallpaper_blur.or(self.wallpaper);
            let (wanted, buffer, first_quad, view) = match (batch.program, batch.texture) {
                (Program::Rect, _) => (RECT, self.rect_vertices.buffer, batch.first_quad, None),
                (Program::Text, _) => {
                    (TEXT, self.tex_vertices.buffer, batch.first_quad, self.atlas)
                }
                (Program::Glass, ImageTexture::Wallpaper | ImageTexture::WallpaperBlur) => {
                    (GLASS, self.tex_vertices.buffer, batch.first_quad, blur)
                }
                (Program::Image, ImageTexture::Wallpaper) => (
                    WALLPAPER,
                    self.tex_vertices.buffer,
                    batch.first_quad,
                    self.wallpaper,
                ),
                (Program::Image, ImageTexture::WallpaperBlur) => {
                    (WALLPAPER, self.tex_vertices.buffer, batch.first_quad, blur)
                }
                (program, texture) => (
                    if program == Program::Glass {
                        GLASS
                    } else {
                        IMAGE
                    },
                    self.tex_vertices.buffer,
                    batch.first_quad,
                    match texture {
                        ImageTexture::WallpaperThumbs => self.wallpaper_thumbs,
                        _ => self.image,
                    },
                ),
            };
            let first_quad = if batch.program == Program::Rect || batch.program == Program::Text {
                first_quad
            } else {
                image_quad_base + first_quad
            };
            if wanted != pipeline {
                device.cmd_bind_pipeline(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipelines[wanted],
                );
                pipeline = wanted;
            }
            if buffer != vertices {
                device.cmd_bind_vertex_buffers(cmd, 0, &[buffer], &[0]);
                vertices = buffer;
            }
            let view = view.map(|t| t.view).or(blank);
            if view != texture {
                if let Some(view) = view {
                    self.push_textures(cmd, view);
                }
                texture = view;
            }
            device.cmd_draw_indexed(cmd, batch.quad_count * 6, 1, 0, (first_quad * 4) as i32, 0);
        }
    }
}

impl Drop for VkRenderer {
    fn drop(&mut self) {
        // SAFETY: nothing is in use once the device is idle; null handles are skipped by Vulkan.
        unsafe {
            let _ = self.device.device_wait_idle();
            if let Some(upload) = self.uploading.take() {
                self.garbage.push(upload.source);
            }
            let textures = [
                self.blank,
                self.atlas,
                self.image,
                self.wallpaper,
                self.wallpaper_blur,
                self.wallpaper_thumbs,
                self.bayer,
            ];
            for tex in textures.into_iter().flatten().chain(self.garbage.drain(..)) {
                self.device.destroy_image_view(tex.view, None);
                self.device.destroy_image(tex.image, None);
                self.device.free_memory(tex.memory, None);
            }
            for buf in [
                &self.staging,
                &self.rect_vertices,
                &self.tex_vertices,
                &self.indices,
            ] {
                self.destroy_buffer(buf);
            }
            for pipeline in self.pipelines {
                self.device.destroy_pipeline(pipeline, None);
            }
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device
                .destroy_descriptor_set_layout(self.set_layout, None);
            self.device.destroy_sampler(self.linear, None);
            self.device.destroy_sampler(self.nearest_repeat, None);
            self.destroy_swapchain_images();
            self.swap_fn.destroy_swapchain(self.swapchain, None);
            self.device.destroy_semaphore(self.acquire, None);
            self.device.destroy_fence(self.fence, None);
            self.device.destroy_command_pool(self.pool, None);
            self.device.destroy_device(None);
            self.surface_fn.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

const COLOR_RANGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::COLOR,
    base_mip_level: 0,
    level_count: 1,
    base_array_layer: 0,
    layer_count: 1,
};

/// An OpenGL scissor box (origin bottom-left) as a Vulkan rect (origin top-left) inside the
/// `width` x `height` surface.
fn to_rect([x, y, w, h]: [i32; 4], width: u32, height: u32) -> vk::Rect2D {
    let (width, height) = (width as i32, height as i32);
    let left = x.clamp(0, width);
    let right = (x + w).clamp(left, width);
    let top = (height - y - h).clamp(0, height);
    let bottom = (height - y).clamp(top, height);
    vk::Rect2D {
        offset: vk::Offset2D { x: left, y: top },
        extent: vk::Extent2D {
            width: (right - left) as u32,
            height: (bottom - top) as u32,
        },
    }
}

/// The overlap of two rects (empty when they do not meet).
fn intersect(a: vk::Rect2D, b: vk::Rect2D) -> vk::Rect2D {
    let (x, y) = (a.offset.x.max(b.offset.x), a.offset.y.max(b.offset.y));
    let right = (a.offset.x + a.extent.width as i32).min(b.offset.x + b.extent.width as i32);
    let bottom = (a.offset.y + a.extent.height as i32).min(b.offset.y + b.extent.height as i32);
    vk::Rect2D {
        offset: vk::Offset2D { x, y },
        extent: vk::Extent2D {
            width: (right - x).max(0) as u32,
            height: (bottom - y).max(0) as u32,
        },
    }
}
