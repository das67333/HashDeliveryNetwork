# HashDeliveryNetwork

Т.к. на данный момент аттрибут #[bench] unstable, а в сторонних крейтах сильно страдает производительность из-за невозможности использования оптимизаций и не удаления dead_code, я решил сделать бенчмарки тестами, включил оптимизации тестов в Cargo.toml и форсировал последовательный запуск бенчмарков.

Тесты запускаются через `cargo test --test tests`, 
бенчмарки через `DISABLE_LOGS=1 cargo test --test benches -- --nocapture`


Сервер может быть запущен с аргументами --ip и --port или без любого из них. В таком случае ip=127.0.0.1, а порт запрашивается у OS.

Поддерживаются следующие типы запросов:


{
    "request_type": "store",
    "key": "SOME_KEY",
    "hash": "SOME_HASH"
}  

{
    "request_type": "load",
    "key": "SOME_KEY"
}  

{
    "request_type: "shutdown"
}
