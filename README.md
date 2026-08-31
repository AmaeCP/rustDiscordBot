# Discord Music Bot (Rust)

Высокопроизводительный Discord музыкальный бот на Rust (**Serenity 0.12 + Poise 0.6 + Songbird 0.6** с поддержкой **Discord DAVE E2EE Voice**).

## Возможности

- Воспроизведение треков с YouTube по ссылке или поисковому запросу.
- Поддержка YouTube плейлистов (мгновенный парсинг без предварительного скачивания).
- Автоматическая очистка ссылок на радио-миксы и личные списки (`list=RD`, `list=LL`, `list=WL`).
- Поддержка пачки ссылок через пробел в одной команде.
- Воспроизведение локальных аудиофайлов с диска сервера (`.mp3`, `.flac`, `.wav`, `.ogg`, `.m4a`).
- Воспроизведение аудиофайлов, прикрепленных прямо в чат Discord (`attachment`).
- Потокобезопасная очередь треков (`VecDeque`) отдельно для каждого сервера.
- Интерактивный прогресс-бар в реальном времени (`/nowplaying`).
- Поддержка паузы, возобновления, пропуска, удаления по номеру и очистки очереди.

---

## Список команд

| Команда | Аргументы | Описание |
|---|---|---|
| `/play` | `query: <ссылка / запрос / пачка ссылок>` | Воспроизводит трек, плейлист или ищет на YouTube. Поддерживает несколько ссылок через пробел |
| `/play_local` | `file: [вложение]`, `path: [путь на диске]` | Воспроизводит прикрепленный в Discord аудиофайл или файл с локального диска хоста |
| `/nowplaying` | — | Показывает текущий трек, автора запроса, время и визуальный прогресс-бар |
| `/queue` | `page: [номер страницы]` | Показывает постраничный список треков в очереди с общей длительностью |
| `/skip` | — | Пропускает текущий трек и включает следующий из очереди |
| `/pause` | — | Ставит текущий трек на паузу |
| `/resume` | — | Снимает воспроизведение с паузы |
| `/remove` | `index: <номер>` | Удаляет трек из очереди по его порядковому номеру (1-based) |
| `/clear` | — | Очищает список предстоящих треков в очереди, не прерывая текущий трек |
| `/stop` | — | Полностью останавливает воспроизведение и очищает всю очередь |
| `/leave` | — | Отключает бота от голосового канала |

---

## Системные требования

- **Rust** (1.75+)
- **FFmpeg** (должен быть доступен в `PATH`)
- **yt-dlp** (должен быть доступен в `PATH`)
- **libopus**

### Установка зависимостей

**macOS (Homebrew):**
```bash
brew install ffmpeg yt-dlp
```

**Ubuntu / Debian:**
```bash
sudo apt update && sudo apt install -y ffmpeg libopus-dev build-essential pkg-config libssl-dev
sudo wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp
sudo chmod a+rx /usr/local/bin/yt-dlp
```

---

## Настройка

1. Создайте файл `.env` в корне проекта:
```env
DISCORD_TOKEN=ваш_токен_бота
```

2. В **[Discord Developer Portal](https://discord.com/developers/applications)**:
   - В разделе **Bot → Privileged Gateway Intents** включите:
     - `Presence Intent`
     - `Server Members Intent`
     - `Message Content Intent`
   - В разделе **OAuth2 → URL Generator**:
     - Scopes: `bot`, `applications.commands`
     - Bot Permissions: `Administrator` (или `Connect`, `Speak`, `Send Messages`)
     - Добавьте бота на ваш сервер.

---

## Запуск

### Локально (режим разработки)
```bash
cargo run
```

### Сборка релизной версии
```bash
cargo build --release
./target/release/discord-music-bot
```

---

## Автозапуск на Linux (Systemd)

Создайте файл сервиса `/etc/systemd/system/discord-bot.service`:

```ini
[Unit]
Description=Discord Music Bot
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/botDiscord
ExecStart=/root/botDiscord/target/release/discord-music-bot
Restart=always
RestartSec=5
EnvironmentFile=/root/botDiscord/.env

[Install]
WantedBy=multi-user.target
```

Управление сервисом:
```bash
sudo systemctl daemon-reload
sudo systemctl enable discord-bot
sudo systemctl start discord-bot
sudo systemctl status discord-bot
```
