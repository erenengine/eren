use ash::vk;

pub fn get_graphic_color_subpass_desc<'a>(
    color_refs: &'a [vk::AttachmentReference2],
) -> vk::SubpassDescription2<'a> {
    vk::SubpassDescription2::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_refs)
}

pub fn get_graphic_depth_subpass_desc<'a>(
    depth_ref: &'a vk::AttachmentReference2,
) -> vk::SubpassDescription2<'a> {
    vk::SubpassDescription2::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .depth_stencil_attachment(depth_ref)
}
