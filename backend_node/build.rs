use std::fs;
use std::path::Path;

fn main() {
    let source = Path::new("index.d.ts");
    let destination = Path::new("src/napi/index.d.ts");

    // Проверяем, существует ли исходный файл
    if !source.exists() {
        panic!("Файл {:?} не найден!", source);
    }

    // Создаем папку, если она отсутствует
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).expect("Не удалось создать директорию");
    }

    // Копируем файл
    fs::rename(source, destination).expect("Ошибка при копировании файла");

    println!("Файл {:?} успешно скопирован в {:?}", source, destination);
}
