# Сборка Sqrilizz Launcher

## Требования

- Rust 1.90 или новее
- Node.js 20 или новее
- pnpm 9 или новее

### Дополнительно для разных платформ

**Windows:**
- Visual Studio Build Tools или Visual Studio с C++ компонентами
- WebView2 (обычно уже установлен в Windows 10/11)

**Linux:**
- webkit2gtk-4.1
- libssl-dev
- libgtk-3-dev
- libayatana-appindicator3-dev
- librsvg2-dev

Установка на Ubuntu/Debian:
```bash
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

**macOS:**
- Xcode Command Line Tools

## Процесс сборки

### 1. Клонирование репозитория

```bash
git clone https://github.com/sqrilizz/SqrilizzLauncher.git
cd SqrilizzLauncher
```

### 2. Установка зависимостей

```bash
pnpm install
```

Первая установка может занять несколько минут.

### 3. Разработка

Запуск в режиме разработки:

```bash
pnpm app:dev
```

Или используйте скрипт (только Linux/macOS):

```bash
./dev.sh
```

Приложение откроется автоматически с hot-reload.

### 4. Сборка релиза

Сборка для текущей платформы:

```bash
pnpm app:build
```

Или напрямую через Tauri:

```bash
cd apps/app
pnpm tauri build
```

### Сборка для конкретных форматов

**Linux - только deb:**
```bash
cd apps/app
pnpm tauri build -- --bundles deb
```

**Linux - deb, RPM и AppImage:**
```bash
cd apps/app
pnpm tauri build -- --bundles deb,rpm,appimage
```

**Windows - NSIS `.exe` и MSI:**
```powershell
cd apps/app
pnpm tauri build -- --bundles nsis,msi
```

**macOS - универсальный бинарник:**
```bash
cd apps/app
pnpm tauri build -- --target universal-apple-darwin
```

## Результаты сборки

Собранные файлы находятся в:
```
apps/app/target/release/bundle/
```

Структура:
- Windows: `nsis/` (установщик) и `msi/` (MSI пакет)
- Linux: `deb/`, `rpm/` и `appimage/`
- macOS: `dmg/` и `macos/`

## Проблемы при сборке

### Долгая первая сборка

Первая сборка компилирует все Rust зависимости и может занять 10-30 минут. Последующие сборки будут намного быстрее.

### Ошибки линковки на Linux

Убедитесь что установлены все системные библиотеки (см. раздел требований).

### Ошибки WebKit на Linux

Если при запуске в dev режиме происходит segfault, используйте workaround:

```bash
export WEBKIT_DISABLE_DMABUF_RENDERER=1
pnpm app:dev
```

Этот workaround уже включен в скрипт `dev.sh`.

### Проблемы с pnpm

Если pnpm не установлен:

```bash
npm install -g pnpm
```

Или используйте corepack (встроен в Node.js 16+):

```bash
corepack enable
corepack prepare pnpm@latest --activate
```

## Очистка

Удаление всех собранных файлов и кэша:

```bash
# Очистка Rust
cargo clean

# Очистка Node.js
rm -rf node_modules
rm -rf apps/*/node_modules
rm -rf packages/*/node_modules

# Переустановка зависимостей
pnpm install
```

## Дополнительная информация

- BUILD_MACOS.md - специфика сборки для macOS
- BUILD_APPIMAGE.md - создание AppImage для Linux
- BUILD_INSTRUCTIONS.md - общие инструкции по сборке
