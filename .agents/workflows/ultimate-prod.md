---
description: Финализация продакшен-релиза, оптимизация сборки, Docker, CI/CD, документация и маркетинговый Launch Kit
---

# /ultimate-prod

Ты — Principal Release Engineer & Growth Lead. Твоя задача — подготовить проект к коммерческому релизу: "$ARGUMENTS" (версия: Release Candidate / Production).

---

## 1. Релизный стресс-тест и валидация
*Применяй навыки:* `.agents/skills/unit-expert`, `integration-expert`, `functional-expert`, `endurance-expert`, `e2e-expert`, `cyber-safety`

1. **Финальный прогон тестов:** Запусти полный набор (`unit`, `integration`, `functional`, `extreme`, `cyber-safety`, `e2e`).
2. **Блокировка релиза:** Если хоть один тест падает или обнаружены уязвимости безопасности — релиз блокируется до применения `/ultimate-fix`.
3. **Очистка артефактов:** Убедись, что временные файлы, дампы и логи удалены из репозитория.

---

## 2. Оптимизация сборки (Production Build)
1. Скомпилируй и собери оптимизированный production-билд в директорию `builds/` (tree-shaking, минификация, удаление отладочных логов).
2. Проверь размер бандла и время первого рендера.

---

## 3. Контейнеризация и Инфраструктура
1. **Dockerfile:** Создай минималистичный, безопасный multi-stage `Dockerfile` (с непривилегированным пользователем `non-root`).
2. **Docker Compose:** Сформируй `docker-compose.yml` (при наличии баз данных или микросервисов).
3. **CI/CD Pipeline:** Настрой `.github/workflows/ci.yml` с автоматическим запуском линтера, тестов и сборки Docker-образа.
4. **Скрипты запуска:** Проверь и актуализируй `scripts/run.ps1` и `scripts/run.sh`.

---

## 4. Документация и версионирование
1. **ROADMAP.md:** Переведи все выполненные задачи текущей версии в статус `[x]`, закрой майлстоун версии.
2. **CHANGELOG.md:** Сформируй чейнджлог по стандарту Keep a Changelog (Added, Changed, Fixed, Security).
3. **README.md:** 
   - Актуализируй бейджи (Build Status, License, Coverage).
   - Проверь Quick Start за 2 шага.
   - Добавь скриншоты или ASCII-диаграмму архитектуры.

---

## 5. Маркетинговый Launch Kit & Дистрибуция
*Применяй навыки:* `.agents/skills/marketing-genius`, `smm-growth-hacker`, `seo-expert`, `brand`, `banner-design`, `slides`

Создай файл `docs/LAUNCH_KIT.md` с готовыми промо-материалами:
1. **Product Hunt / Hacker News (Show HN):** Питч с акцентом на проблему, решение, стек и ссылку.
2. **Reddit (r/programming, r/webdev, r/SideProject):** Нативный технический пост о разработке, архитектуре `core/ui` и инсайтах.
3. **Telegram / X (Twitter):** Вирусный короткий тред с визуальной демонстрацией фич и призывом к действию (CTA).
4. **OpenGraph & SEO:** Проверка фавиконов, `og:image`, мета-тегов заголовков и описаний.

---

## 6. Финальный релизный отчет
Выведи в чат сводную карточку релиза:
* **Версия:** Номер релиза (например, `v1.0.0-prod`).
* **Статус тестов:** 100% Passed.
* **Артефакты сборки:** `builds/`, `Dockerfile`, `.github/workflows/ci.yml`.
* **Готовность Launch Kit:** `docs/LAUNCH_KIT.md` сформирован.