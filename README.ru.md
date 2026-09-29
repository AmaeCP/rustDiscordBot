# Discord Music Bot

**Языки:** [English](README.md) | Русский

Музыкальный бот для Discord на Rust с использованием Serenity, Poise и Songbird.

## Возможности

- Воспроизведение ссылок YouTube, поисковых запросов, плейлистов и нескольких ссылок за один вызов.
- Воспроизведение аудиофайла, прикреплённого к сообщению, или файла, доступного на хосте бота.
- Отдельная очередь для каждого сервера с автоматическим запуском следующего трека.
- Повтор трека или очереди, перемешивание, регулировка громкости, перемотка и история воспроизведения.
- Публичная панель с кнопками: предыдущий трек, пауза/продолжение, следующий, стоп, очистка, очередь, повтор, перемешивание и громкость.
- На сервере действует одна актуальная панель. При управлении из другого текстового канала панель переносится, старая отключается.
- Восстановление после кика бота из voice: прерванный трек и очередь очищаются; бот ждёт нового `/play`.
- Если треков нет, бот выходит из voice через 10 минут. `/leave` отключает сразу.
- Пока играет трек или очередь не пуста, приоритет у текущего voice-канала. Во время пустого ожидания отключения новый `/play` может переместить бота в канал вызывающего.

Состояние воспроизведения и очередь хранятся в памяти и теряются при перезапуске бота.

## Команды

| Команда | Аргументы | Описание |
|---|---|---|
| `/play` | `query` | Воспроизвести ссылку YouTube, поисковый запрос, плейлист или несколько ссылок. |
| `/play_local` | `file` или `path` | Воспроизвести вложенный аудиофайл или файл на хосте бота. Путь указывается на сервере, где запущен бот. |
| `/nowplaying` | — | Показать текущий трек и прогресс воспроизведения. |
| `/queue` | `page` (необязательно) | Показать текущий трек и страницу очереди. |
| `/skip` | — | Пропустить текущий трек. |
| `/previous` | — | Вернуться к предыдущему треку из истории. |
| `/pause`, `/resume` | — | Поставить воспроизведение на паузу или продолжить. |
| `/loop` | `mode`: `Off`, `Track` или `Queue` (необязательно) | Задать или переключить режим повтора. |
| `/shuffle` | — | Перемешать следующие треки. |
| `/volume` | `level`: `0–200` | Задать громкость в процентах. |
| `/seek` | `position` | Перемотать на секунды или время `мм:сс` (например, `90` или `1:30`). |
| `/replay` | — | Запустить текущий трек сначала. |
| `/remove` | `index` | Удалить трек по позиции в очереди, начиная с 1. |
| `/clear` | — | Очистить следующие треки, не прерывая текущий. |
| `/stop` | — | Остановить воспроизведение и очистить очередь; бот выйдет после таймера простоя. |
| `/leave` | — | Немедленно отключиться от voice и сбросить состояние воспроизведения. |

Команды зарегистрированы как slash-команды. Функции также объявляют поддержку префиксных команд Poise, но префикс сообщений в проекте не настроен — используйте slash-команды и кнопки панели.

## Требования

- Rust и Cargo
- `yt-dlp` и FFmpeg, доступные через `PATH`
- `libopus` и стандартные инструменты сборки платформы

### macOS

```bash
brew install ffmpeg yt-dlp opus
```

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev ffmpeg yt-dlp libopus-dev
```

## Настройка и запуск

1. Создайте приложение и бота в [Discord Developer Portal](https://discord.com/developers/applications), затем скопируйте токен.
2. Создайте `.env` в корне проекта:

   ```env
   DISCORD_TOKEN=токен_бота
   ```

3. В **Bot → Privileged Gateway Intents** включите **Presence**, **Server Members** и **Message Content**. Сейчас приложение запрашивает все Gateway Intents.
4. Пригласите бота с OAuth2 scopes `bot` и `applications.commands`. Выдайте нужные разрешения в каналах: View Channels, Send Messages, Embed Links, Connect и Speak. Administrator не требуется.
5. Запустите бота или соберите release-версию:

   ```bash
   cargo run
   # или
   cargo build --release
   ./target/release/discord-music-bot
   ```

## Автозапуск через systemd

Настройте пути и пользователя под свой VPS. Пример соответствует установке в `/root/rustDiscordBot`:

Этот пример повторяет текущий unit на VPS и запускает бота от `root`. Для новой production-установки лучше создать отдельного системного пользователя и дать ему доступ только к каталогу проекта.

```ini
[Unit]
Description=Discord Music Bot
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/rustDiscordBot
ExecStart=/root/rustDiscordBot/target/release/discord-music-bot
Restart=always
RestartSec=5
EnvironmentFile=/root/rustDiscordBot/.env

[Install]
WantedBy=multi-user.target
```

Сохраните файл как `/etc/systemd/system/discord-bot.service`, затем соберите и запустите сервис:

```bash
cd /root/rustDiscordBot
cargo build --release
sudo systemctl daemon-reload
sudo systemctl enable --now discord-bot.service
sudo systemctl status discord-bot.service
```

Логи сервиса: `sudo journalctl -u discord-bot.service -f`.

## Проверки

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
