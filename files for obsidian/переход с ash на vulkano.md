# Почему?
1. Vulkano - преследует идеологию самого rust о безопасном использовании api
2. Имеет [книжку](https://vulkano.rs/01-introduction/01-introduction.html) по изучению, которая часто обновляется 
# Начнём
файл cargo.toml:
```rust
[dependencies]
vulkano = "0.34.0" //новая зависимость для интеграции библиотеки

[profile.dev]
opt-level = 1 //уровень оптимизации компиляции
```

не забудь выполнить 
```bash 
cargo clean
```
и удалить директорию `target/debug`
чтобы избавиться от старых зависимостей внутри компилятора

дальнейший код, необходим для проверки возможности инициализации Vulkan на устройстве:
```rust
#![allow(unused)]  
use vulkano::instance::{Instance, InstanceCreateInfo};  
use vulkano::VulkanLibrary;  
fn main() {  
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");  
    let instance =  
        Instance::new(library, InstanceCreateInfo::default()).expect("failed to create instance");  
}
```
при запуске данного кода создаётся и ожидается экземпляр интерфейса Vulkan, при запуске его
```bash
cargo run
```
не должно возникать ошибок

выбор физического устройства:
```rust
let physical_device = instance  
    .enumerate_physical_devices()  
    .expect("could not enumerate devices")  
    .next()  
    .expect("no devices available");
```
В компьютере может быть несколько физических(и даже программных) устройств  поддерживающих выполнение инструкций vulkan api, потому мы создаём пронумерованный список устройств (автоматически первым в нём будет самое мощное устройство), далее функция `next()` выбирает из списка первый пункт и присваивает это значение переменной `physical_device`, есть нет подходящих устройств - вывод ошибки. Возможно что список окажется пустым, так как сам vulkan установлен, но ни одно устройство его не поддерживает. В таком случае необходимо обратить внимание на драйвера или спецификацию устройств.

Очереди:
Иными словами поток направляемый на устройство. Разные семейства очередей имеют разные функции, которые могут быть и схожими и полностью разными, по типу тут только графика, тут только вычисления и тд (подробнее о конкретных семьях на конкретных моделях устройств на официальном [сайте](http://vulkan.gpuinfo.org/))
Перечисление всех семей очередей на данном физическом девайсе:
```rust
for family in physical_device.queue_family_properties() {
    println!("Found a queue family with {:?} queue(s)", family.queue_count);
}
```

добавим несколько зависимостей:
```rust
use vulkano::instance::{Instance, InstanceCreateInfo};  
use vulkano::VulkanLibrary;
```
рассмотрим что у нас за очереди существуют и найдём индекс того, в тегах которого указанна работа с графикой:
```rust
let queue_family_index = physical_device  
    .queue_family_properties()  
    .iter()  
    .enumerate()  
    .position(|(_queue_family_index, queue_family_properties)| {  
        queue_family_properties  
            .queue_flags  
            .contains(QueueFlags::GRAPHICS)  
    })  
    .expect("couldn't find a graphical queue family") as u32;  
```
теперь создадим логическое устройство с выбранными параметрами семейства очереди на конкретном девайсе:
```rust
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
```
Данный код возвращает переменные `device` и `queues`, c первым всё понятно, это наше логическое устройство, а вот очереди это итерируемый объект внутри которого несколько потоков обращающихся к нашему логическому устройству. Потому мы должны создать уже просто очередь (`queue`) которая будет первой из списка `queues` 
```rust
let queue = queues.next().unwarp();
```
тут (да и везде далее) метод `.unwarp()` это обработчик результата. То есть если мы получили первую очередь из списка очередей то мы получили её с индификатором `Ok()`, в таком случае `unwarp()` преобразует его просто в объект и присваивает значение переменной. Если же очереди пусты - то нам вернётся индификатор `Err`, тогда же `unwarp()` вызовет `panic()` и аварийно завершит работу программы. 