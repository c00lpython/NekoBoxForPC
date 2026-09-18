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
    pub fn find_default_core_path() -> Option<PathBuf> {
        let candidates = [
            "sing-box.exe",
            "singbox.exe",
            "singbox/Windows/singbox.exe",
            "../sing-box.exe",
            "../singbox/Windows/singbox.exe",
            "singbox/Linux/singbox",
            "singbox",
        ];

        for c in candidates {
            let path = PathBuf::from(c);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Запускает ядро sing-box с сгенерированной Ultimate-конфигурацией.
    pub async fn start(&self, config: Value) -> Result<(), String> {
        let runtime_dir = Path::new(".runtime");
        let _ = tokio::fs::create_dir_all(runtime_dir).await;
        let config_path = runtime_dir.join("active_singbox_config.json");

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

        // Создаем каталог logs/
        let logs_dir = Path::new("logs");
        let _ = tokio::fs::create_dir_all(logs_dir).await;

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
