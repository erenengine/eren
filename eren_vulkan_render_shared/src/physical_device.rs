use std::{collections::HashSet, ffi::CStr, sync::Arc};

use ash::vk;
use thiserror::Error;

use crate::{
    instance::{DeviceCreationError, Instance, PhysicalDevicesEnumerationError},
    surface::{Surface, SurfaceInfo},
};

pub fn get_required_physical_device_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::default().shader_clip_distance(true)
}

fn has_required_features(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
    let features = instance.get_physical_device_features(physical_device);
    if features.shader_clip_distance != vk::TRUE {
        return false;
    }
    true
}

pub fn get_required_physical_device_extensions() -> Vec<&'static std::ffi::CStr> {
    let mut required_extensions = vec![ash::khr::swapchain::NAME];

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        required_extensions.push(ash::khr::portability_subset::NAME);
    }

    required_extensions
}

fn has_required_extensions(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
    let extensions = instance.get_physical_device_extension_properties(physical_device);
    for required_ext_name_cstr in get_required_physical_device_extensions().iter() {
        let required_ext_name = unsafe { CStr::from_ptr(required_ext_name_cstr.as_ptr()) };
        let found = extensions.iter().any(|ext| {
            let available_ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            available_ext_name == required_ext_name
        });
        if !found {
            return false;
        }
    }
    true
}

#[derive(Debug)]
struct QueueFamilyIndices {
    graphics_queue_family_index: Option<u32>,
    compute_queue_family_index: Option<u32>,
    transfer_queue_family_index: Option<u32>,
    sparse_binding_queue_family_index: Option<u32>,
    present_queue_family_index: Option<u32>,
}

fn find_queue_family_indices(
    instance: &Instance,
    surface: &Surface,
    physical_device: vk::PhysicalDevice,
) -> QueueFamilyIndices {
    // 후보를 저장해 두기 위한 변수들
    let mut graphics_with_present = None;
    let mut graphics_only = None;

    let mut compute_dedicated = None;
    let mut compute_any = None;

    let mut transfer_dedicated = None;
    let mut transfer_any = None;

    let mut sparse_binding = None;
    let mut present_only = None;

    // 모든 큐 패밀리 정보를 가져온다
    let queue_families = instance.get_physical_device_queue_family_properties(physical_device);
    for (index, qf) in queue_families.iter().enumerate() {
        let idx = index as u32;
        let flags = qf.queue_flags;

        // 서피스 프레젠트 지원 여부
        let present_supported = surface.can_queue_family_present_to_surface(physical_device, idx);

        /* -------- 그래픽스 -------- */
        if flags.contains(vk::QueueFlags::GRAPHICS) {
            // 그래픽스 + 프레젠트 겸용이면 최우선
            if present_supported && graphics_with_present.is_none() {
                graphics_with_present = Some(idx);
            } else if graphics_only.is_none() {
                graphics_only = Some(idx);
            }
        }

        /* -------- 컴퓨트 -------- */
        if flags.contains(vk::QueueFlags::COMPUTE) {
            // 그래픽스 플래그가 없으면 전용 컴퓨트
            if !flags.contains(vk::QueueFlags::GRAPHICS) && compute_dedicated.is_none() {
                compute_dedicated = Some(idx);
            }
            if compute_any.is_none() {
                compute_any = Some(idx);
            }
        }

        /* -------- 트랜스퍼 -------- */
        if flags.contains(vk::QueueFlags::TRANSFER) {
            let dedicated = !flags.intersects(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE);
            if dedicated && transfer_dedicated.is_none() {
                transfer_dedicated = Some(idx);
            }
            if transfer_any.is_none() {
                transfer_any = Some(idx);
            }
        }

        /* -------- Sparse Binding -------- */
        if flags.contains(vk::QueueFlags::SPARSE_BINDING) && sparse_binding.is_none() {
            sparse_binding = Some(idx);
        }

        /* -------- Present Only -------- */
        if present_supported && !flags.contains(vk::QueueFlags::GRAPHICS) && present_only.is_none()
        {
            present_only = Some(idx);
        }
    }

    /* -------- 실제 선택 로직 -------- */

    // 그래픽스: 프레젠트를 겸하는 큐가 있으면 최우선, 없으면 그래픽스만
    let graphics_index = graphics_with_present.or(graphics_only);

    // 프레젠트: 그래픽스 큐가 이미 지원하면 그대로, 아니면 present-only
    let present_index = graphics_with_present.or(present_only);

    // 컴퓨트: 전용 → 아무 컴퓨트 → 그래픽스
    let compute_index = compute_dedicated
        .or_else(|| {
            // 그래픽스 큐와 다른 큐면 더 좋으므로 우선 선택
            if compute_any != graphics_index {
                compute_any
            } else {
                None
            }
        })
        .or(graphics_index);

    // 트랜스퍼: 전용 → 아무 트랜스퍼(그래픽스/컴퓨트와 다른 큐 선호) → 그래픽스
    let transfer_index = transfer_dedicated
        .or_else(|| {
            if transfer_any != graphics_index && transfer_any != compute_index {
                transfer_any
            } else {
                None
            }
        })
        .or(graphics_index);

    QueueFamilyIndices {
        graphics_queue_family_index: graphics_index,
        compute_queue_family_index: compute_index,
        transfer_queue_family_index: transfer_index,
        sparse_binding_queue_family_index: sparse_binding,
        present_queue_family_index: present_index,
    }
}

#[derive(Debug)]
struct PhysicalDeviceCandidate {
    physical_device: vk::PhysicalDevice,
    queue_family_indices: QueueFamilyIndices,
    surface_info: SurfaceInfo,
    score: u32,
}

// 점수를 기반으로 가장 좋은 물리 디바이스를 선택하는 함수
fn pick_best_physical_device(
    instance: &Instance,
    surface: &Surface,
) -> Result<Option<PhysicalDeviceCandidate>, PhysicalDevicesEnumerationError> {
    let mut best_candidate: Option<PhysicalDeviceCandidate> = None;

    let devices = instance.get_physical_devices()?;

    for device in devices {
        // 필수 조건: 필수 기능과 확장이 지원되어야 함
        if !has_required_features(&instance, device) || !has_required_extensions(&instance, device)
        {
            continue;
        }

        let indices = find_queue_family_indices(instance, surface, device);

        // 필수 조건: 그래픽과 프레젠트 큐가 존재해야 함
        if indices.graphics_queue_family_index.is_none()
            || indices.present_queue_family_index.is_none()
        {
            continue;
        }

        let surface_info = match surface.query_surface_info(device) {
            Ok(surface_info) => surface_info,
            Err(e) => {
                log::error!("Failed to query surface info: {}", e);
                continue;
            }
        };

        // 필수 조건: swapchain 생성 가능해야 함
        if !surface_info.can_create_swapchain() {
            continue;
        }

        // 디바이스 속성 확인
        let props = instance.get_physical_device_properties(device);

        // 기본 점수: 디스크리트 GPU면 1000점
        let mut score = match props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 10,
        };

        // 기타 점수 보정 (예: transfer 큐가 따로 있으면 추가 점수)
        if let Some(transfer_index) = indices.transfer_queue_family_index {
            if Some(transfer_index) != indices.graphics_queue_family_index {
                score += 100; // 전용 transfer queue 있음
            }
        }

        // sparse binding 지원 시 약간 가산
        if indices.sparse_binding_queue_family_index.is_some() {
            score += 10;
        }

        let candidate = PhysicalDeviceCandidate {
            physical_device: device,
            queue_family_indices: indices,
            surface_info,
            score,
        };

        if let Some(ref current_best) = best_candidate {
            if candidate.score > current_best.score {
                best_candidate = Some(candidate);
            }
        } else {
            best_candidate = Some(candidate);
        }
    }

    Ok(best_candidate)
}

#[derive(Debug, Error)]
pub enum PhysicalDeviceInitializationError {
    #[error("Physical device initialization error: {0}")]
    PhysicalDevicesEnumerationError(#[from] PhysicalDevicesEnumerationError),

    #[error("No compatible physical device found")]
    NoCompatiblePhysicalDevice,
}

pub struct PhysicalDevice {
    instance: Arc<Instance>,
    handle: vk::PhysicalDevice,
    queue_family_indices: QueueFamilyIndices,
    surface_info: SurfaceInfo,
}

impl PhysicalDevice {
    pub fn new(
        instance: Arc<Instance>,
        surface: &Surface,
    ) -> Result<Self, PhysicalDeviceInitializationError> {
        let best_candidate = pick_best_physical_device(&instance, surface)?;

        log::debug!("Best physical device: {:#?}", best_candidate);

        if let Some(candidate) = best_candidate {
            Ok(Self {
                instance,
                handle: candidate.physical_device,
                queue_family_indices: candidate.queue_family_indices,
                surface_info: candidate.surface_info,
            })
        } else {
            Err(PhysicalDeviceInitializationError::NoCompatiblePhysicalDevice)
        }
    }

    pub fn get_queue_infos(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut unique_indices = HashSet::new();
        let mut queue_infos = Vec::new();
        let queue_priority = &[1.0f32];

        let all_indices = [
            self.queue_family_indices.graphics_queue_family_index,
            self.queue_family_indices.compute_queue_family_index,
            self.queue_family_indices.transfer_queue_family_index,
            self.queue_family_indices.present_queue_family_index,
        ];

        for index_opt in all_indices.iter().copied().flatten() {
            if unique_indices.insert(index_opt) {
                queue_infos.push(
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(index_opt)
                        .queue_priorities(queue_priority),
                );
            }
        }

        queue_infos
    }

    pub fn create_device(
        &self,
        info: vk::DeviceCreateInfo,
    ) -> Result<ash::Device, DeviceCreationError> {
        self.instance.create_device(self.handle, info)
    }
}
