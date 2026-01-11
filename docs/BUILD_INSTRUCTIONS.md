# Sqrilizz Launcher - Build Instructions

## 📦 Сборка для Linux (уже выполнено)

Готовые пакеты находятся в `target/release/bundle/`:

- ✅ `deb/Sqrilizz Launcher_1.1.0_amd64.deb` - для Debian/Ubuntu
- ✅ `rpm/Sqrilizz Launcher-1.1.0-1.x86_64.rpm` - для Fedora/RHEL
- ✅ `appimage/Sqrilizz Launcher_1.1.0_amd64.AppImage` - универсальный

---

## 🪟 Сборка для Windows

### Шаг 1: Установка зависимостей

#### 1.1 Установить Node.js

- Скачать с https://nodejs.org/ (рекомендуется LTS версия)
- Установить с настройками по умолчанию

#### 1.2 Установить Rust

- Скачать с https://rustup.rs/
- Запустить `rustup-init.exe`
- Выбрать вариант установки по умолчанию (1)
- После установки перезапустить терминал

#### 1.3 Установить pnpm

Открыть PowerShell или CMD и выполнить:

```bash
npm install -g pnpm
```

#### 1.4 Установить Visual Studio Build Tools (если еще не установлено)

- Скачать с https://visualstudio.microsoft.com/downloads/
- Выбрать "Build Tools for Visual Studio"
- При установке выбрать "Desktop development with C++"

### Шаг 2: Подготовка проекта

#### 2.1 Скопировать проект на Windows

Перенести всю папку проекта на Windows машину

#### 2.2 Открыть терминал в папке проекта

- Открыть PowerShell или CMD
- Перейти в папку проекта:

```bash
cd путь\к\KrutoyLauncherZOV
```

#### 2.3 Установить зависимости

```bash
pnpm install
```

Это займет несколько минут.

### Шаг 3: Сборка

Выполнить команду сборки:

```bash
pnpm run app:build
```

Процесс займет 10-15 минут. После завершения установщики будут в:

```
target\release\bundle\msi\Sqrilizz Launcher_1.1.0_x64_en-US.msi
target\release\bundle\nsis\Sqrilizz Launcher_1.1.0_x64-setup.exe
```

---

## 🚀 Быстрая сборка (если все уже установлено)

### Linux:

```bash
pnpm run app:build
```

### Windows:

```powershell
pnpm run app:build
```

---

## 📝 Примечания

### Размеры файлов (примерно):

- Linux DEB: ~80-100 MB
- Linux AppImage: ~100-120 MB
- Windows MSI: ~80-100 MB
- Windows NSIS: ~80-100 MB

### Требования к системе для сборки:

- **RAM**: минимум 4 GB, рекомендуется 8 GB
- **Диск**: минимум 10 GB свободного места
- **Интернет**: для загрузки зависимостей

### Возможные проблемы:

#### Windows: "error: linker `link.exe` not found"

**Решение**: Установить Visual Studio Build Tools (см. Шаг 1.4)

#### Windows: "ENOENT: no such file or directory"

**Решение**: Запустить PowerShell от имени администратора

#### Linux/Windows: "Out of memory"

**Решение**: Закрыть другие программы, увеличить swap (Linux)

#### Ошибка при `pnpm install`

**Решение**:

```bash
# Очистить кэш
pnpm store prune
# Удалить node_modules
rm -rf node_modules
# Установить заново
pnpm install
```

---

## 🔧 Разработка

### Запуск в режиме разработки:

#### Терминал 1 - Фронтенд:

```bash
pnpm run web:dev --filter=@modrinth/app-frontend
```

#### Терминал 2 - Приложение:

```bash
pnpm run app:dev
```

---

## 📦 Итоговые файлы

После сборки на обеих платформах у вас будет:

### Linux:

- `Sqrilizz Launcher_1.1.0_amd64.deb`
- `Sqrilizz Launcher-1.1.0-1.x86_64.rpm`
- `Sqrilizz Launcher_1.1.0_amd64.AppImage`

### Windows:

- `Sqrilizz Launcher_1.1.0_x64_en-US.msi`
- `Sqrilizz Launcher_1.1.0_x64-setup.exe`

---

## 📞 Поддержка

- Telegram: https://t.me/sqrilizz
- Website: https://sqrilizz.xyz
- GitHub: https://github.com/sqrilizz/SqrilizzLauncher

---

**Версия**: 1.1.0
**Дата**: 28.12.2025
