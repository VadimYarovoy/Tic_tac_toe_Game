# TIC_TAC_TOE discord bot
## Участники проекта
- Группа 3530904/00104
1. [Яровой Вадим](https://github.com/VadimYarovoy)
2. [Дорошин Данил](https://github.com/ddddanil)

## используемые инструменты:
- [Rust](https://doc.rust-lang.ru/book/) 1.66.1
- [Cargo](https://doc.rust-lang.org/cargo/) 1.66.1
- [serenity](https://docs.rs/serenity/latest/serenity/) - Rust library for the Discord API
- [image](https://docs.rs/image/latest/image/) - provides native rust implementations of image encoding and decoding as well as some basic image manipulation functions

## Пример работы:
![image](https://user-images.githubusercontent.com/89383982/213172306-2a6793c5-6993-4435-88fb-90e5cf7323d9.png)

## Инструкция по сборке:
- установить rust [Rust](https://www.rust-lang.org/ru/tools/install)
- создать бота на [сайте](https://discord.com/developers/applications)
- во вкладке OAuth2/URLgenerator в первом меню выбрать `bot` 
  во втором `admin` (можно более тоно настроить разрешения предоставленные боту)
- после копируем ссылку и добавляем бота на сервер
- во вкладке OAuth2/general копируем токен и добавляем его в файл `token.txt`
- клонируем репозиторий 
```
git clone git@github.com:VadimYarovoy/Tic_tac_toe_Game.git
```
- добавляем файл `token.txt` в корень проэкта
- собираем и запускаем бота
 ```
 cargo run
 ```
  может потребоваться некоторое время на скачивавние всех зависимостей

## Поддерживаемые команды:
|описание| команда|
|-|-|
|проверка того, что бот запустился |`/ping!`|
|встать в очередь на игру              |`/start`|
|завершить сессию / выйти из очереди   | `/stop`|
