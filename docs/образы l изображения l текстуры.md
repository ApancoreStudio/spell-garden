код для этой главы ищи тут [[рисуем vulkan-ом]] 
##### `Важно понимать, что текстура это частный случай двумерного изображения, а изображения=образ и имеет размерности от 1 до 3, в свою очередь изображение является частным случаем доступа GPU к данным (да-да, тоже самое, что и буферы, которые мы рассматривали тут)`[[переход с ash на vulkano]]
Для более понятного описание далее будет использоваться термин "изображение" с пометкой размерности
Образы могут быть представлены в жёстко заданных форматах.
#### Свойства изображения:
1. Существует два типа трёхмерных изображений: собственно трёхмерные изображения и массивы двумерных слоёв. Разница в том, что в первом случае слои должны быть непрерывными, а во втором можно управлять слоями по отдельности, как если бы они были отдельными двумерными изображениями.
2. При создании изображений надо выбрать формат его пикселей. В зависимости от формата пиксели изображения могут содержать от одного до четырёх компонентов. Другими словами, каждый пиксель представляет собой массив из одного-четырёх значений. Четыре компонента называются R, G, B и A. Описания компонентов в качестве кодировки RGBA является условным. Потому в "красном пикселе" можно хранить не только значение красного цвета
3. [`список форматов`](https://docs.rs/vulkano/0.34.0/vulkano/format/enum.Format.html) (формат может иметь от 1 до 4 компанет)
4. Примеры форматов:
	1. `R8_SINT`
	2. `A2R10G10B10_SSCALED_PACK32`
	3. `B10G11R11_UFLOAT_PACK32` - пачка из 32 бит, где первая десятка это голубой, за ним с 11 по 22 бит это зелёный, и с 23 по 32 это красный. При этом каждый из них это положительное число с плавающей запятой.
#### Создание изображения:
Как и буферы, изображения создаются путём предоставления информации об изображении и его распределении. Однако, в отличие от буферов, изображения всегда создаются в неинициализированном состоянии.
```rust
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::format::Format;

let image = Image::new(
    memory_allocator.clone(),
    ImageCreateInfo {
        image_type: ImageType::Dim2d,
        format: Format::R8G8B8A8_UNORM,
        extent: [1024, 1024, 1],
        usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC,
        ..Default::default()
    },
    AllocationCreateInfo {
        memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
        ..Default::default()
    },
)
.unwrap();
```
Мы передаём размеры изображения и желаемый формат. Как и в случае с буферами, изображения также необходимо создавать с флагами, которые описывают, как будет использоваться изображение. Если использовать его не так, как было указано при создании, возникнет ошибка.
#### Очистка изображения:
В отличие от буферов, изображения имеют непрозрачную структуру памяти, зависящую от реализации. Это означает, что нельзя изменить изображение, напрямую записывая данные в его память. Не существует такого понятия, как `CpuAccessibleImage`. Таким образом, единственный способ прочитать или записать изображение — это попросить графический процессор сделать это. Именно это мы и собираемся сделать, попросив графический процессор заполнить наше изображение определённым цветом. Это называется _очисткой_ изображения.
```rust
use vulkano::command_buffer::ClearColorImageInfo;
use vulkano::format::ClearColorValue;

let mut builder = AutoCommandBufferBuilder::primary(
    &command_buffer_allocator,
    queue.queue_family_index(),
    CommandBufferUsage::OneTimeSubmit,
)
.unwrap();

builder
    .clear_color_image(ClearColorImageInfo {
        clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
        ..ClearColorImageInfo::image(image.clone())
    })
    .unwrap();

let command_buffer = builder.build().unwrap();
```
#### Нормализованные компоненты:
Перечисление `ClearColorValue` enum указывает, каким цветом заполнить изображение. В зависимости от формата изображения мы должны использовать правильный вариант перечисления `ClearValue`.
Здесь мы передаём значения с плавающей запятой, потому что изображение было создано в формате `R8G8B8A8_UNORM`. Часть `R8G8B8A8`. означает, что четыре компонента хранятся по 8 бит каждый, а суффикс `UNORM`. означает «беззнаковое нормализованное». «Нормализованные» координаты означают, что их значение в памяти (в диапазоне от 0 до 255) интерпретируется как значения с плавающей запятой. Значение в памяти `0`. интерпретируется как значение с плавающей запятой `0.0`. и значение в памяти `255`. интерпретируется как значение с плавающей запятой `1.0`.
При использовании любого формата, суффикс которого — `UNORM` (а также `SNORM` и `SRGB`), все операции, выполняемые с изображением (за исключением копирования в память), рассматривают изображение так, как если бы оно содержало значения с плавающей запятой. Именно поэтому мы передаём `[0.0, 0.0, 1.0, 1.0]`. Значения `1.0` фактически будут храниться в памяти как `255`.
#### Экспорт содержимого изображения:
Тут мы попробуем взять наше изображение буфер и преобразовать его в понятный человеку стандарт png
Очевидно, что нам надо создать буфер дабы забрать данные из графического процессора и провернуть над ним действия:
```rust
let buf = Buffer::from_iter(
    memory_allocator.clone(),
    BufferCreateInfo {
        usage: BufferUsage::TRANSFER_DST,
        ..Default::default()
    },
    AllocationCreateInfo {
        memory_type_filter: MemoryTypeFilter::PREFER_HOST
            | MemoryTypeFilter::HOST_RANDOM_ACCESS,
        ..Default::default()
    },
    (0..1024 * 1024 * 4).map(|_| 0u8),
)
.expect("failed to create buffer");
```
Размер буфера - это количество "пикселей" на количество битов в нем. В нашем случае это `1024*1024*4`

Теперь изменим конструктор изображения, который раньше только чистил изображение, заливая его цветом. Нам необходимо теперь дать ему возможность копировать изображение в созданный буфер:
```rust
use vulkano::command_buffer::CopyImageToBufferInfo;

builder
    .clear_color_image(ClearColorImageInfo {
        clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
        ..ClearColorImageInfo::image(image.clone())
    })
    .unwrap()
    .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(
        image.clone(),
        buf.clone(),
    ))
    .unwrap();
```
Важно понять, что наш буфер это уже не изображение и 255 *не будет* интерпретироваться как 1.0
Снова надо сделать объект `future` с методом `.wait()` как было разобрано ранее в [[переход с ash на vulkano]]
```rust
use vulkano::sync::{self, GpuFuture};

let future = sync::now(device.clone())
    .then_execute(queue.clone(), command_buffer)
    .unwrap()
    .then_signal_fence_and_flush()
    .unwrap();

future.wait(None).unwrap();
```
Далее добавим библиотеку работы с изображениями как зависимость cargo:
```rust
image = "0.24"
```
далее учимся работать с методом обработки изображения:
```rust
use image::{ImageBuffer, Rgba};

let buffer_content = buf.read().unwrap();
let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
```
наконец сохраняем всё в файл и добавляем уведомление самому себе о том, что всё хорошо:
```rust
image.save("image.png").unwrap();

println!("Everything succeeded!");
```
