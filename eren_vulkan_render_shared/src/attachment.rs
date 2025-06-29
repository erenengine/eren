use ash::vk;

pub struct Attachment {
    pub desc: vk::AttachmentDescription2<'static>,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}
