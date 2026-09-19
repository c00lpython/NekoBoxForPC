//! Асинхронный супервайзер процесса ядра sing-box и интеграция с Metacubexd.

use serde_json::Value;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex};

/// Кольцевой буфер логов для оперативного просмотра последних N строк в UI.
pub struct LogRingBuffer {
    capacity: usize,
    buffer: VecDeque<String>,
}

impl LogRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, line: String) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(line);
    }

    pub fn get_all(&self) -> Vec<String> {
        self.buffer.iter().cloned().collect()
    }
}

/// Асинхронный супервайзер процесса прокси-ядра.
pub struct CoreSupervisor {
    core_binary_path: PathBuf,
    child: Arc<Mutex<Option<Child>>>,
    log_buffer: Arc<Mutex<LogRingBuffer>>,
    log_sender: broadcast::Sender<String>,
}

impl CoreSupervisor {
    pub fn new(core_path: Option<PathBuf>) -> Result<Self, String> {
        let path = if let Some(p) = core_path {
            p
        } else {
            Self::find_default_core_path()
                .ok_or_else(|| "Бинарный файл ядра sing-box не найден".to_string())?
        };

        let (log_sender, _) = broadcast::channel(500);

        Ok(Self {
            core_binary_path: path,
            child: Arc::new(Mutex::new(None)),
            log_buffer: Arc::new(Mutex::new(LogRingBuffer::new(500))),
            log_sender,
        })
    }

    /// Автоматический поиск бинарника ядра singbox.
    ///
    /// Порядок поиска:
    /// 1. Текущий каталог и подкаталоги (legacy пути)
    /// 2. Каталог рядом с исполняемым файлом `nbpfpc`
    /// 3. Системный PATH (`where` на Windows, `which` на Unix)
    /// 4. Стандартные системные каталоги ОС
    /// Автоматический поиск исполняемого файла ядра sing-box по цепочке приоритетов:
    /// 1. Каталог исполняемого файла nbpfpc (высший приоритет - ядро рядом с приложением)
    /// 2. Относительные пути от текущего рабочего каталога (singbox.exe, singbox/Windows/singbox.exe и др.)
    /// 3. Системный PATH (`where` на Windows, `which` на Unix)
    /// 4. Стандартные системные каталоги ОС
    pub fn find_default_core_path() -> Option<PathBuf> {
        let exe_names: &[&str] = if cfg!(windows) {
            &["singbox.exe", "sing-box.exe"]
        } else {
            &["singbox", "sing-box"]
        };

        // 1. Каталог исполняемого файла nbpfpc (наивысший приоритет: ядро "внутри" дистрибутива)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                for name in exe_names {
                    let candidate = exe_dir.join(name);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
                // Подкаталог singbox/ рядом с exe
                let platform_subdir = if cfg!(windows) {
                    if Self::is_windows_7() {
                        "Windows7"
                    } else {
                        "Windows"
                    }
                } else if cfg!(target_os = "macos") {
                    "MacOS"
                } else {
                    "Linux"
                };
                let subdir_candidate = exe_dir
                    .join("singbox")
                    .join(platform_subdir)
                    .join(if cfg!(windows) { "singbox.exe" } else { "singbox" });
                if subdir_candidate.exists() {
                    return Some(subdir_candidate);
                }
            }
        }

        // 2. Относительные пути (legacy) — от текущего рабочего каталога
        let relative_candidates = [
            "singbox.exe",
            "sing-box.exe",
            "singbox/Windows/singbox.exe",
            "singbox/Windows7/singbox.exe",
            "../singbox.exe",
            "../sing-box.exe",
            "../singbox/Windows/singbox.exe",
            "../singbox/Windows7/singbox.exe",
            "singbox/Linux/singbox",
            "singbox/MacOS/singbox",
            "singbox",
        ];

        for c in relative_candidates {
            let path = PathBuf::from(c);
            if path.exists() {
                return Some(path);
            }
        }

        // 3. Поиск в системном PATH
        if let Some(found) = Self::find_in_system_path(exe_names) {
            return Some(found);
        }

        // 4. Стандартные системные каталоги
        let system_dirs = Self::get_system_search_dirs();
        for dir in system_dirs {
            for name in exe_names {
                let candidate = dir.join(name);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// Поиск исполняемого файла в системном PATH.
    fn find_in_system_path(names: &[&str]) -> Option<PathBuf> {
        if let Ok(path_var) = std::env::var("PATH") {
            let separator = if cfg!(windows) { ';' } else { ':' };
            for dir_str in path_var.split(separator) {
                let dir = PathBuf::from(dir_str);
                for name in names {
                    let candidate = dir.join(name);
                    if candidate.exists() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }

    /// Стандартные каталоги для поиска ядра на разных ОС.
    fn get_system_search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        #[cfg(windows)]
        {
            // Windows стандартные пути
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                dirs.push(PathBuf::from(&local_app_data).join("NekoBox"));
                dirs.push(PathBuf::from(&local_app_data).join("NekoBoxPlus"));
            }
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                dirs.push(PathBuf::from(&program_files).join("NekoBox"));
                dirs.push(PathBuf::from(&program_files).join("NekoBoxPlus"));
            }
        }

        #[cfg(not(windows))]
        {
            // Unix стандартные пути
            dirs.push(PathBuf::from("/usr/local/bin"));
            dirs.push(PathBuf::from("/opt/nekobox"));
            if let Ok(home) = std::env::var("HOME") {
                dirs.push(PathBuf::from(&home).join(".local/bin"));
                dirs.push(PathBuf::from(&home).join(".nekobox"));
            }
        }

        dirs
    }

    /// Определяет, запущен ли процесс на Windows 7 (для авто-выбора legacy бинарника).
    ///
    /// Возвращает `true` на Windows 7/Server 2008 R2 (NT 6.1).
    /// На non-Windows всегда `false`.
    pub fn is_windows_7() -> bool {
        #[cfg(windows)]
        {
            // Проверка через переменную окружения — простой и надёжный способ
            if let Ok(ver) = std::env::var("OS") {
                if ver != "Windows_NT" {
                    return false;
                }
            }
            // Проверяем winver через `ver` command output или registry.
            // Простой эвристический подход: если нет bcryptprimitives.dll в System32,
            // предполагаем Win7.
            let system32 = std::env::var("SystemRoot")
                .unwrap_or_else(|_| "C:\\Windows".to_string());
            let bcrypt_path = PathBuf::from(&system32)
                .join("System32")
                .join("bcryptprimitives.dll");
            // На Windows 8+ этот файл всегда есть, на Win7 — нет
            !bcrypt_path.exists()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Определяет, запущен ли процесс на 32-битной архитектуре (x86).
    pub fn is_x86_arch() -> bool {
        cfg!(target_arch = "x86")
    }

    /// Запускает ядро sing-box с сгенерированной Ultimate-конфигурацией.
    pub async fn start(&self, config: Value) -> Result<(), String> {
        let data_dir = crate::storage::get_data_dir();
        let _ = tokio::fs::create_dir_all(&data_dir).await;
        let config_path = data_dir.join("active_singbox_config.json");

        let config_str = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Ошибка сериализации конфига: {}", e))?;
        tokio::fs::write(&config_path, config_str)
            .await
            .map_err(|e| format!("Ошибка записи файла конфига: {}", e))?;

        self.start_file(&config_path).await
    }

    /// Запускает ядро sing-box с указанным файлом конфигурации (.json или .conf).
    pub async fn start_file(&self, config_path: &Path) -> Result<(), String> {
        let mut child_guard = self.child.lock().await;
        if child_guard.is_some() {
            return Err("Ядро уже запущено".into());
        }

        // Создаем каталог logs/ внутри data/
        let logs_dir = crate::storage::get_data_dir().join("logs");
        let _ = tokio::fs::create_dir_all(&logs_dir).await;

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session_log_path = logs_dir.join(format!("session_{}.log", now_secs));
        let latest_log_path = logs_dir.join("latest.log");

        // Подготавливаем канал для асинхронного логирования в файлы
        let (file_tx, mut file_rx) = tokio::sync::mpsc::channel::<String>(1000);

        let initial_header = format!(
            "[{}] === SingBox Session Started ===\nBinary: {}\nConfig: {}\n------------------------------------------------------------\n",
            now_secs,
            self.core_binary_path.display(),
            config_path.display()
        );

        let s_path = session_log_path.clone();
        let l_path = latest_log_path.clone();
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            let mut s_file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&s_path)
                .await
                .ok();

            let mut l_file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&l_path)
                .await
                .ok();

            if let Some(ref mut f) = s_file {
                let _ = f.write_all(initial_header.as_bytes()).await;
                let _ = f.flush().await;
            }
            if let Some(ref mut f) = l_file {
                let _ = f.write_all(initial_header.as_bytes()).await;
                let _ = f.flush().await;
            }

            while let Some(line) = file_rx.recv().await {
                let formatted = format!("{}\n", line);
                let bytes = formatted.as_bytes();
                if let Some(ref mut f) = s_file {
                    let _ = f.write_all(bytes).await;
                    let _ = f.flush().await;
                }
                if let Some(ref mut f) = l_file {
                    let _ = f.write_all(bytes).await;
                    let _ = f.flush().await;
                }
            }
        });

        let mut cmd = Command::new(&self.core_binary_path);
        cmd.arg("run")
            .arg("-c")
            .arg(config_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child_proc = cmd
            .spawn()
            .map_err(|e| format!("Не удалось запустить процесс ядра: {}", e))?;

        let stdout = child_proc.stdout.take();
        let stderr = child_proc.stderr.take();

        // Запуск фонового сбора логов stdout
        if let Some(out) = stdout {
            let buffer = self.log_buffer.clone();
            let sender = self.log_sender.clone();
            let ftx = file_tx.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(out).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let mut buf = buffer.lock().await;
                    buf.push(line.clone());
                    let _ = sender.send(line.clone());
                    let _ = ftx.send(line).await;
                }
            });
        }

        // Запуск фонового сбора логов stderr
        if let Some(err) = stderr {
            let buffer = self.log_buffer.clone();
            let sender = self.log_sender.clone();
            let ftx = file_tx.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(err).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let formatted = format!("[STDERR] {}", line);
                    let mut buf = buffer.lock().await;
                    buf.push(formatted.clone());
                    let _ = sender.send(formatted.clone());
                    let _ = ftx.send(formatted).await;
                }
            });
        }

        *child_guard = Some(child_proc);
        Ok(())
    }

    /// Проверяет жизнеспособность ядра через опрос Clash API.
    pub async fn wait_for_ready(&self, timeout: Duration) -> bool {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .unwrap_or_default();

        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            // Если процесс упал, не ждем весь таймаут
            if !self.is_running().await {
                return false;
            }
            if client.get("http://127.0.0.1:9090/version").send().await.is_ok() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }

        false
    }

    /// Останавливает ядро и освобождает сетевой адаптер TUN.
    pub async fn stop(&self) -> Result<(), String> {
        let mut child_guard = self.child.lock().await;
        if let Some(mut child_proc) = child_guard.take() {
            let _ = child_proc.kill().await;
            let _ = child_proc.wait().await;
        }
        Ok(())
    }

    /// Проверяет, запущено ли ядро в данный момент.
    pub async fn is_running(&self) -> bool {
        let mut child_guard = self.child.lock().await;
        if let Some(ref mut child_proc) = *child_guard {
            match child_proc.try_wait() {
                Ok(None) => true,
                _ => {
                    *child_guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Подписка на поток живых логов (для трансляции во Flutter / Dart Streams).
    pub fn subscribe_logs(&self) -> broadcast::Receiver<String> {
        self.log_sender.subscribe()
    }

    /// Получение снимка последних строк логов.
    pub async fn get_log_snapshot(&self) -> Vec<String> {
        self.log_buffer.lock().await.get_all()
    }
}
