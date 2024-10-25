#![allow(unused)]
use anyhow::Result;
use ash::{
    self,
    vk::{self, DeviceQueueCreateInfo},
};
fn main() -> Result<()> {
    let entry = unsafe { ash::Entry::load() }?;

    let instance = {
        let application_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_0);
        let create_info = vk::InstanceCreateInfo::builder().application_info(&application_info);
        unsafe { entry.create_instance(&create_info, None) }?
    };

    let dev = {
        let queue_priorities = [1.0];
        let queue_create_infos = [DeviceQueueCreateInfo::builder()
            .queue_family_index(0)
            .queue_priorities(&queue_priorities)
            .build()];
        let create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_create_infos);
        let ph_dev = unsafe { instance.enumerate_physical_devices() }?;
        unsafe { instance.create_device(ph_dev[0], &create_info, None) }?
    };
    println!("ABOBUS");
    let queue = unsafe { dev.get_device_queue(0, 0) };

    let com_pool = {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(0);
        unsafe { dev.create_command_pool(&create_info, None) }?
    };

    let com_buf = {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(com_pool)
            .command_buffer_count(1);
        unsafe { dev.allocate_command_buffers(&create_info) }?
            .into_iter()
            .next()
            .unwrap()
    };

    unsafe { dev.destroy_command_pool(com_pool, None) };
    unsafe { dev.destroy_device(None) };
    unsafe { instance.destroy_instance(None) };
    Ok(())
}
