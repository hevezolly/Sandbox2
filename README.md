# Sandbox

небольшая песочница с различными материалами и их взаимодействиями
![sand](https://user-images.githubusercontent.com/28929816/175764779-acd05b39-9805-4231-9e72-506e83d08215.gif)
![water](https://user-images.githubusercontent.com/28929816/175764797-0f727b20-c8f5-40f9-9fe8-8e3a722934f0.gif)


Проект сделан с нуля. Для визуализации используются крейты [winit](https://crates.io/crates/winit) и [pixels](https://crates.io/crates/pixels)


В проекте присутствует определенный уровень оптимизации: мир поделен на чанки, внутри каждого чанка обновляются пиксели внутри dirty rect. Каждый чанк обновляется в отдельном потоке

![engine](https://user-images.githubusercontent.com/28929816/175765029-dd032896-8acf-4067-bc94-9e774c62f94b.gif)
