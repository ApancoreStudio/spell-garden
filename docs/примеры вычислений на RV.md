###### *Тут будет продолжение описание работы над рендером с использованием технологий Rust + Vulkano*

Попробуем что-то да вычислить. Не просто же так у нас есть графический процессор. Что насчёт работы с большим объёмом данных?
```rust
use std::sync::Arc;  
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};  
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};  
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};  
use vulkano::instance::{Instance, InstanceCreateInfo};  
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};  
use vulkano::pipeline::compute::ComputePipelineCreateInfo;  
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;  
use vulkano::pipeline::{ComputePipeline, PipelineLayout, PipelineShaderStageCreateInfo};  
use vulkano::sync::{self, GpuFuture};  
use vulkano::VulkanLibrary;  
use vulkano::pipeline::Pipeline;  
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};  
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;  
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};  
use vulkano::pipeline::PipelineBindPoint;  
  
fn main() {  
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");  
    let instance =  
        Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");  
  
    let physical_device = instance  
        .enumerate_physical_devices()  
        .expect("could not enumerate devices")  
        .next()  
        .expect("no devices available");  
  
    let queue_family_index = physical_device  
        .queue_family_properties()  
        .iter()  
        .enumerate()  
        .position(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))  
        .expect("couldn't find a graphical queue family") as u32;  
  
    let (device, mut queues) = Device::new(  
        physical_device,  
        DeviceCreateInfo {  
            queue_create_infos: vec![QueueCreateInfo {  
                queue_family_index,  
                ..Default::default()  
            }],  
            ..Default::default()  
        },  
    )  
    .expect("failed to create device");  
  
    let queue = queues.next().unwrap();  
  
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));  
  
    let data_iter = 0..65536u32;  
    let data_buffer = Buffer::from_iter(  
        memory_allocator.clone(),  
        BufferCreateInfo {  
            usage: BufferUsage::STORAGE_BUFFER,  
            ..Default::default()  
        },  
        AllocationCreateInfo {  
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE  
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,  
            ..Default::default()  
        },  
        data_iter,  
    )  
    .expect("failed to create buffer");  
  
    mod cs {  
        vulkano_shaders::shader! {  
            ty: "compute",  
            src: r"  
				#version 460

				layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

				layout(set = 0, binding = 0) buffer Data {
				    uint data[];
				} buf;

				void main() {
				    uint idx = gl_GlobalInvocationID.x;
					buf.data[idx] *= 12;
				}
	        ",  
        }  
    }  
    let shader = cs::load(device.clone()).expect("failed to create shader module");  
    let cs = shader.entry_point("main").unwrap();  
    let stage = PipelineShaderStageCreateInfo::new(cs);  
    let layout = PipelineLayout::new(  
        device.clone(),  
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])  
            .into_pipeline_layout_create_info(device.clone())  
            .unwrap(),  
    )  
    .unwrap();  
    let compute_pipeline = ComputePipeline::new(  
        device.clone(),  
        None,  
        ComputePipelineCreateInfo::stage_layout(stage, layout),  
    )  
    .expect("failed to create compute pipeline");  
    let descriptor_set_allocator =  
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());  
    let pipeline_layout = compute_pipeline.layout();  
    let descriptor_set_layouts = pipeline_layout.set_layouts();  
  
    let descriptor_set_layout_index = 0;  
    let descriptor_set_layout = descriptor_set_layouts  
        .get(descriptor_set_layout_index)  
        .unwrap();  
    let descriptor_set = PersistentDescriptorSet::new(  
        &descriptor_set_allocator,  
        descriptor_set_layout.clone(),  
        [WriteDescriptorSet::buffer(0, data_buffer.clone())],
        [],  
    )  
    .unwrap();  
  
    let command_buffer_allocator = StandardCommandBufferAllocator::new(  
        device.clone(),  
        StandardCommandBufferAllocatorCreateInfo::default(),  
    );  
    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(  
        &command_buffer_allocator,  
        queue.queue_family_index(),  
        CommandBufferUsage::OneTimeSubmit,  
    )  
        .unwrap();  
    let work_group_counts = [1024, 1, 1];  
    command_buffer_builder  
        .bind_pipeline_compute(compute_pipeline.clone())  
        .unwrap()  
        .bind_descriptor_sets(  
            PipelineBindPoint::Compute,  
            compute_pipeline.layout().clone(),  
            descriptor_set_layout_index as u32,  
            descriptor_set,  
        )  
        .unwrap()  
        .dispatch(work_group_counts)  
        .unwrap();  
    let command_buffer = command_buffer_builder.build().unwrap();  
    let future = sync::now(device.clone())  
        .then_execute(queue.clone(), command_buffer)  
        .unwrap()  
        .then_signal_fence_and_flush()  
        .unwrap();  
    future.wait(None).unwrap();  
    let content = data_buffer.read().unwrap();  
    for (n, val) in content.iter().enumerate() {  
        assert_eq!(*val, n as u32 * 12);  
    }  
    println!("Everything succeeded!");  
    println!("data: {}", content[16]);  
}
```
Страшно? А то! Но мы попробуем разобраться

#### кусок кода с 75 строчку по 85 мы разберём тут -> [[шейдеры]]

### всё до 55 строчки описывалось ранее

```rust
    let data_iter = 0..65536u32;  
    let data_buffer = Buffer::from_iter(  
        memory_allocator.clone(),  
        BufferCreateInfo {  
            usage: BufferUsage::STORAGE_BUFFER,  
            ..Default::default()  
        },  
        AllocationCreateInfo {  
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE  
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,  
            ..Default::default()  
        },  
        data_iter,  
    )  
    .expect("failed to create buffer"); 
```
как в ранее описанных документах - тут мы создаём оговоренный итерируемый буфер

```rust
    mod cs {  
        vulkano_shaders::shader! {  
            ty: "compute",  
            src: r" [далее текст шейдера]
```
думаю тут стоит пояснить. Мы создаём карту "cs" или же culculate shader, в которую входит конструктор шейдера. Где поля `ty` - имя, `src` - источник, ключ `r` - чтение того что находится далее. Это может быть как отдельный файл, так и текст шейдера в формате строки

```rust 
let shader = cs::load(device.clone()).expect("failed to create shader module");
```
тут всё просто, загружаем наш шейдер в девайс(необходим именно клон, так как по правилам работы памяти в rust при передаче его как оригинал - после выполнения сборщик мусора уберёт его из стека).
```rust
let stage = PipelineShaderStageCreateInfo::new(cs);  
    let layout = PipelineLayout::new(  
        device.clone(),  
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])  
            .into_pipeline_layout_create_info(device.clone())  
            .unwrap(),  
    )  
    .unwrap();  
    let compute_pipeline = ComputePipeline::new(  
        device.clone(),  
        None,  
        ComputePipelineCreateInfo::stage_layout(stage, layout),  
    )  
    .expect("failed to create compute pipeline");  
```
Тут уже сложнее. Мы создаём описание вычислительной операции и создаём объект вычислительного конвейера. На данном этапе, мы просим Vulkan автоматически сгенерировать макет, однако, стоит учитывать, что в дальнейшем желательно описать его в ручную, для увеличения производительности. 
```rust
let pipeline_layout = compute_pipeline.layout();  
    let descriptor_set_layouts = pipeline_layout.set_layouts();  
  
    let descriptor_set_layout_index = 0;  
    let descriptor_set_layout = descriptor_set_layouts  
        .get(descriptor_set_layout_index)  
        .unwrap();  
    let descriptor_set = PersistentDescriptorSet::new(  
        &descriptor_set_allocator,  
        descriptor_set_layout.clone(),  
        [WriteDescriptorSet::buffer(0, data_buffer.clone())],
        [],  
    )  
    .unwrap();  
```
Тут мы, по аналогу с буферами, создаём управление памятью аллакатором. Далее создаём дескриптор и указываем ему тот макет, на который он ориентирован. 
Важно:
```rust        
[WriteDescriptorSet::buffer(0, data_buffer.clone())],
[],  
```
тут мы берём именно нулевой макет, так как это первый вычислительный проход, и он является ключевым, поскольку последующие могут зависеть от последующих, потому макет 0 прохода является наиболее общим для GPU
```RUST
    let work_group_counts = [1024, 1, 1];  
    command_buffer_builder  
        .bind_pipeline_compute(compute_pipeline.clone())  
        .unwrap()  
        .bind_descriptor_sets(  
            PipelineBindPoint::Compute,  
            compute_pipeline.layout().clone(),  
            descriptor_set_layout_index as u32,  
            descriptor_set,  
        )  
        .unwrap()  
        .dispatch(work_group_counts)  
        .unwrap();  
```
далее всё очень просто, мы создаём предполагаемое количество проходов нашей рабочей группы (а точнее количество рабочих групп) и создаём командный буфер с помощью конструктора, прикрепляя к нему всё описание созданное раньше.

### Время рисовать фракталы
код -> [[фрактальный шейдер]] 

вычислительный шейдер:
```c
#version 460

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

    vec2 z = vec2(0.0, 0.0);
    float i;
    for (i = 0.0; i < 1.0; i += 0.005) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }

    vec4 to_write = vec4(vec3(i), 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}
```

Разбор построчно:
```c
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
```
Дабы запаралелить вычисление тут, лучше всего будет обрабатывать по пикселю за один вызов `main()`. Потому мы используем значение `8x8x1` для содержания информации об конкретном пикселе, который сейчас в обработке.
```c
layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
```
Тут мы объявляем о том, что мы будем работать с двухмерным изображением. Значение `set = 0` и `binding = 0` означает, что наше изображение будет храниться в нулевом слоте из нулевого набора дескрипторов.  Очевидно, что несовпадение формата вернёт нам ошибку.
```c
vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
```
Здесь логика происходящего немного теряется, но я постараюсь всё объяснить:
Для начала `gl_GlobalInvocationID.xy` - это обращение к значению индекса пикселя, который на данный момент находиться в обработке. ключевые слова `vec2` эквивалентны записи в Rust `[f32;2]`. Вектор `c` это значение нашего пикселя к комплексной плоскости (почему нам это надо? ну мы ведь рисуем множество Мандельброта, там каждый пиксель изображения определяет свой цвет в зависимости от значения его комплексного значения). Нормированная координата, в свою очередь, это значение координаты нашего пикселя в условных единицах относительно центра изображения. 
```c
vec2 z = vec2(0.0, 0.0);
float i;
for (i = 0.0; i < 1.0; i += 0.005) {
    z = vec2(
        z.x * z.x - z.y * z.y + c.x,
        z.y * z.x + z.x * z.y + c.y
    );

    if (length(z) > 4.0) {
        break;
    }
}
```
Теперь мы хотим выяснить, находится ли комплексное число, с которым мы работаем (то есть `c`), в множестве Мандельброта. Согласно определению множества Мандельброта, число `c` находится в множестве, если функция `f(z) = z² + c` расходится при повторении от `z = 0` (`z` — комплексное число).
Отсюда, нам важно лишь понять, насколько сильно расходиться число. А значит - мы берём отсюда лишь значение `i`, ведь если число не расходиться то `i = 1.0`, в противном случае `i > 1.0`. 
```c
vec4 to_write = vec4(vec3(i), 1.0);
imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
```
Наконец, мы записываем (путём преобразования `vec2->vec3->vec4`)  в изображение (`ivec2 ~ [i32;2]`) значение цвета и яркость пикселя

На этом описание шейдера окончено, далее обработаем и вызовем его в нашем коде, а так же опишем рабочую область (само изображение).