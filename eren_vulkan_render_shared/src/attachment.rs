use ash::vk;

pub struct Attachment<'a> {
    pub desc: vk::AttachmentDescription2<'a>,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}
