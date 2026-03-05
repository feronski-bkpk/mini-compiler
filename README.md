# Mini Compiler

**Минимальный компилятор C-подобного языка, написанный на Rust с полной поддержкой LL(1) грамматики и восстановления после ошибок.**

[![Rust](https://img.shields.io/badge/rust-2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-97%20passed-blue)]()
[![LL(1)](https://img.shields.io/badge/grammar-LL(1)-purple)]()
[![Sprint](https://img.shields.io/badge/sprint-2%20complete-success)]()

## Оглавление

- [Особенности](#особенности)
- [Структура проекта](#структура-проекта)
- [Установка и сборка](#установка-и-сборка)
- [Makefile команды](#makefile-команды)
- [Быстрый старт](#быстрый-старт)
- [Использование CLI](#использование-cli)
- [Демонстрации](#демонстрации)
- [Визуализация AST](#визуализация-ast)
- [Тестирование](#тестирование)
- [Документация](#документация)
- [Архитектура](#архитектура)
- [LL(1) анализ](#ll1-анализ)
- [Восстановление после ошибок](#восстановление-после-ошибок)
- [Команда](#команда)
- [Полезные ссылки](#полезные-ссылки)

## Особенности

### Реализовано
- **Лексический анализатор** - полное распознавание 30+ типов токенов
- **Препроцессор** - удаление комментариев, макросы, условная компиляция
- **Точное позиционирование** - строка:колонка для всех ошибок
- **Восстановление после ошибок** - продолжение анализа после ошибок
- **Поддержка комментариев** - `//` и `/* */` (включая вложенные)
- **Escape-последовательности** - `\n`, `\t`, `\r`, `\\`, `\"`, `\'`
- **Формальная грамматика** - полная EBNF спецификация языка
- **Парсер с рекурсивным спуском** - LL(1) грамматика
- **AST (Abstract Syntax Tree)** - полная иерархия узлов
- **Поддержка функций** - с возвращаемыми типами (`->`)
- **Поддержка структур** - с доступом к полям (`.`)
- **Инкременты и декременты** - префиксные и постфиксные (`++x`, `x++`, `--x`, `x--`)
- **Приоритет операторов** - правильная обработка 9 уровней
- **Визуализация AST** - текстовый, DOT и JSON форматы
- **LL(1) анализ** - вычисление First/Follow множеств и проверка грамматики
- **Метрики ошибок** - качество восстановления, предотвращение каскадных ошибок
- **Обработка ошибок** - детальные сообщения с предложениями по исправлению

### Технические характеристики
- **Язык**: Rust 2024 edition
- **Обработка ошибок**: Детальные сообщения с восстановлением
- **Поддержка кодировок**: UTF-8
- **Окончания строк**: Unix (`\n`) и Windows (`\r\n`)
- **Тесты**: 97+ тестов (unit + интеграционные + LL(1))

## Структура проекта

```
mini-compiler/
├── src/                          # Исходный код
│   ├── common/                   # Общие типы данных
│   │   ├── mod.rs                # Утилиты
│   │   ├── token.rs              # Токены
│   │   └── position.rs           # Позиция в исходном коде
│   ├── lexer/                    # Лексический анализатор
│   │   ├── mod.rs                # Основной модуль
│   │   ├── scanner.rs            # Сканер (основная логика)
│   │   └── error.rs              # Ошибки лексического анализа
│   ├── parser/                   # Парсер
│   │   ├── mod.rs                # Экспорт модуля
│   │   ├── parser.rs             # Рекурсивный спуск
│   │   ├── ast.rs                # Структуры AST
│   │   ├── error.rs              # Ошибки парсера с метриками
│   │   ├── visitor.rs            # Паттерн Visitor
│   │   ├── pretty_printer.rs     # Текстовый вывод AST
│   │   ├── dot_generator.rs      # Graphviz DOT генератор
│   │   ├── json_generator.rs     # JSON генератор
│   │   ├── ll1.rs                # LL(1) анализ (First/Follow)
│   │   ├── error_productions.rs  # Продукции для ошибок
│   │   └── grammar.txt           # Формальная грамматика
│   ├── preprocessor/             # Препроцессор
│   │   ├── mod.rs                # Основной модуль
│   │   ├── macros.rs             # Таблица макросов
│   │   └── error.rs              # Ошибки препроцессора
│   ├── utils/                    # Вспомогательные функции
│   ├── lib.rs                    # Точка входа библиотеки
│   └── main.rs                   # Точка входа CLI
├── tests/                        # Тестовые файлы
│   ├── lexer/                    # Тесты лексера
│   ├── parser/                   # Тесты парсера
│   │   ├── valid/                # Корректные программы
│   │   ├── invalid/              # Некорректные программы
│   │   └── ast_output/           # Ожидаемые результаты
│   ├── lexer_tests.rs            # 7 тестов лексера
│   ├── parser_tests.rs           # 27 тестов парсера
│   ├── preprocessor_tests.rs     # 8 тестов препроцессора
│   ├── integration_tests.rs      # 2 интеграционных теста
│   └── ll1_tests.rs              # 2 теста LL(1) анализа
├── examples/                     # Демонстративные файлы
│   ├── hello.src                 # Простая программа
│   ├── factorial.src             # Рекурсивный факториал
│   ├── struct.src                # Работа со структурами
│   ├── increments.src            # Демонстрация инкрементов
│   └── errors.src                # Файл с ошибками для демонстрации
├── docs/                         # Документация
│   ├── CHECKLIST.md              # Чек-лист по спринтам
│   ├── language_spec.md          # Спецификация языка
│   └── grammar.md                # Формальная грамматика
├── Cargo.toml                    # Конфигурация Cargo
├── Cargo.lock                    # Версии зависимостей
├── Makefile                      # Система сборки
├── README.md                     # Этот файл
└── .gitignore                    # Игнорируемые файлы Git
```

## Установка и сборка

### Предварительные требования

- **Rust 1.70 или новее**
- **Cargo** (менеджер пакетов Rust)
- **Git** (для клонирования репозитория)
- **Graphviz** (опционально, для визуализации AST)

### Установка

```bash
# Клонировать репозиторий
git clone https://github.com/feronski-bkpk/mini-compiler.git
cd mini-compiler

# Собрать проект
cargo build --release

# Проверить сборку
cargo run -- --version
```

### Базовые команды сборки

```bash
# Проверка компиляции
cargo check

# Форматирование кода
cargo fmt

# Проверка стиля кода
cargo clippy

# Сборка
cargo build
cargo build --release
```

## Makefile команды

Проект включает полный набор Makefile целей для всех операций:

### Основные команды
```bash
make build         # Сборка проекта
make release       # Сборка в режиме релиза
make check         # Проверка кода
make clean         # Очистка
```

### Тестирование
```bash
make test                    # Все тесты
make test-lexer              # Тесты лексера
make test-parser             # Тесты парсера
make test-preprocessor       # Тесты препроцессора
make test-integration        # Интеграционные тесты
make test-ll1                # LL(1) тесты
make test-common             # Тесты общих модулей
make test-doc                # Документационные тесты
make test-all                # Все тесты (включая LL(1))
```

### Демонстрации
```bash
make ast-demo                # Визуализация AST
make inc-demo                # Демонстрация инкрементов
make error-demo              # Восстановление после ошибок
make ll1-demo                # LL(1) анализ грамматики
make full-pipeline           # Полный пайплайн
make example                 # Примеры использования CLI
```

### Качество кода
```bash
make lint                    # Проверка линтером
make fmt                     # Форматирование кода
make fmt-check               # Проверка форматирования
```

### Документация
```bash
make docs                    # Генерация документации
make docs-private            # Документация с приватными элементами
```

### Анализ и утилиты
```bash
make udeps                   # Неиспользуемые зависимости
make bench                   # Бенчмарки
make coverage                # Покрытие кода
make graphviz-check          # Проверка Graphviz
make install-deps            # Установка зависимостей
make create-test-files       # Создание тестовых файлов
make help                    # Справка по Makefile
```

## Быстрый старт

### 1. Быстрый запуск всех демонстраций

```bash
# Создать тестовые файлы
make create-test-files

# Собрать проект
make build

# Запустить все демонстрации
make ast-demo
make inc-demo
make error-demo
make ll1-demo
make full-pipeline
```

### 2. Полный пайплайн компиляции вручную

```bash
# Создайте тестовый файл с инкрементами
cat > examples/demo.src << 'EOF'
#define MAX 100
#define DEBUG 1

fn main() {
    int x = 5;
    x++;                    // Постфиксный инкремент
    ++x;                    // Префиксный инкремент
    int y = x++ + ++x;      // Смешанное использование
    
    #ifdef DEBUG
        int debug_var = y;
    #endif
    
    return y;
}
EOF

# Шаг 1: Препроцессор
cargo run -- preprocess --input examples/demo.src --output examples/processed.src --show

# Шаг 2: Лексический анализ
cargo run -- lex --input examples/processed.src --verbose

# Шаг 3: Синтаксический анализ
cargo run -- parse --input examples/processed.src --ast-format text

# Шаг 4: Полный пайплайн одной командой
cargo run -- full --input examples/demo.src --ast-format dot --output ast.dot --show-metrics
```

## Использование CLI

### Команды CLI

```bash
# Информация
cargo run -- info                    # Базовая информация
cargo run -- info --verbose          # Подробная информация
cargo run -- spec                    # Спецификация языка

# Лексический анализ
cargo run -- lex --input file.src
cargo run -- lex --input file.src --format json
cargo run -- lex --interactive

# Синтаксический анализ
cargo run -- parse --input file.src
cargo run -- parse --input file.src --ast-format dot --output ast.dot
cargo run -- parse --input file.src --show-metrics

# Препроцессор
cargo run -- preprocess --input file.src --output processed.src --show
cargo run -- preprocess --input file.src --defines "DEBUG=1" "VERSION=2"

# Полный пайплайн
cargo run -- full --input file.src --ast-format text
cargo run -- full --input file.src --show-metrics

# Проверка синтаксиса
cargo run -- check --input file.src
cargo run -- check --input file.src --strict

# Специальные демонстрации
cargo run -- inc-demo                
cargo run -- error-demo --input examples/errors.src
cargo run -- ll1 --show-first --show-follow

# Тестирование
cargo run -- test
```

### Форматы вывода

```bash
# Форматы для лексического анализа
--format text      # Человекочитаемый текст
--format json      # JSON формат
--format minimal   # Только ошибки
--format verbose   # Подробный вывод

# Форматы для AST
--ast-format text  # Текстовый AST с отступами
--ast-format dot   # Graphviz DOT для визуализации
--ast-format json  # JSON для машинной обработки
```

## Демонстрации

### Демонстрация инкрементов
```bash
make inc-demo
# или
cargo run -- inc-demo
```

Показывает:
- Разницу между префиксными и постфиксными операторами
- Лексический анализ с подсчетом токенов `++` и `--`
- Построение AST с инкрементами
- Примеры выражений: `x++ + ++x`

### Демонстрация восстановления после ошибок
```bash
make error-demo
# или
cargo run -- error-demo --input examples/errors.src --max-errors 50
```

Показывает:
- Множественные синтаксические ошибки
- Предложения по исправлению
- Метрики ошибок (total, actual, cascading)
- AST построен несмотря на ошибки

### LL(1) анализ грамматики
```bash
make ll1-demo
# или
cargo run -- ll1 --show-first --show-follow
```

Показывает:
- First множества для всех нетерминалов
- Follow множества
- Проверка LL(1) свойств
- Подтверждение корректности грамматики

### Полный пайплайн
```bash
make full-pipeline
# или
cargo run -- full --input examples/full_demo.src --show-metrics
```

## Визуализация AST

### Текстовый формат
```bash
cargo run -- parse --input examples/factorial.src
```

Пример вывода с инкрементами:
```
Program [line 1]:
  FunctionDecl: main -> int [line 1]:
    Parameters: []
    Body [line 1]:
      Block [line 2-8]:
        VarDecl: int x = 5
        Expr: (x++)
        Expr: (++x)
        VarDecl: int y = (x++) + (++z)
        Return: y
```

### Graphviz DOT формат
```bash
# Генерация DOT файла
cargo run -- parse --input examples/struct.src --ast-format dot --output ast.dot

# Визуализация (требуется Graphviz)
dot -Tpng ast.dot -o ast.png

# Узлы раскрашены по типам:
# - Функции: синий
# - Переменные: зеленый  
# - Выражения: оранжевый
# - Литералы: желтый
```

### JSON формат
```bash
# Для машинной обработки
cargo run -- parse --input examples/hello.src --ast-format json --output ast.json
```

## Тестирование

### Запуск тестов

```bash
# Все тесты
cargo test

# С подробным выводом
cargo test -- --nocapture

# Конкретные модули (через Makefile)
make test-lexer
make test-parser
make test-preprocessor
make test-ll1
make test-integration
make test-common
make test-all
```

### Примеры тестов

```rust
// Тест инкрементов
#[test]
fn test_increment_decrement() {
    assert_parses("
        fn main() {
            int x = 5;
            x++;
            ++x;
            x--;
            --x;
            int y = x++ + ++z;
        }
    ");
}

// Тест восстановления после ошибок
#[test]
fn test_error_recovery_missing_semicolon() {
    let output = parse_string("fn main() { return 42 }");
    assert!(output.has_errors());
    assert!(output.ast.is_some());
}
```

## Документация

### Генерация документации

```bash
# Локальная документация
cargo doc --open

# Документация с приватными элементами
cargo doc --document-private-items --open

# С помощью Make
make docs
make docs-private
```

### Ключевые документы

1. **Спецификация языка** - `docs/language_spec.md`
   - Полная грамматика в EBNF
   - Типы данных и операторы
   - Операторы инкремента/декремента
   - Примеры программ

2. **Формальная грамматика** - `docs/grammar.md`
   - Детальное описание всех правил
   - Приоритет операторов (10 уровней)
   - EBNF нотация
   - LL(1) свойства

## Архитектура

### Компоненты системы

```
     Исходный код
          │
          ▼
   ┌──────────────┐
   │ Препроцессор │  ← #define, #ifdef, комментарии
   └──────────────┘
          │
          ▼
   ┌──────────────┐
   │   Лексер     │  ← Токенизация
   └──────────────┘
          │
          ▼
   ┌──────────────┐
   │   Парсер     │  ← Рекурсивный спуск (LL(1))
   └──────────────┘
          │
          ├──────────▶ AST
          │
          ├──────────▶ Визуализация (text/dot/json)
          │
          ├──────────▶ Метрики ошибок
          │
          └──────────▶ LL(1) анализ
```

### Ключевые компоненты

#### Препроцессор (`src/preprocessor/`)
- Удаление комментариев
- Подстановка макросов
- Условная компиляция

#### Лексический анализатор (`src/lexer/`)
- 32+ типа токенов
- Точное позиционирование
- Восстановление после ошибок

#### Парсер (`src/parser/`)
- Рекурсивный спуск с LL(1) грамматикой
- Полное AST с поддержкой инкрементов
- 3 стратегии восстановления после ошибок

## LL(1) анализ

### First и Follow множества

```rust
// Пример вычисления First множеств
First(E) = { "(", "id" }
First(E') = { "+", "ε" }
First(T) = { "(", "id" }
First(T') = { "*", "ε" }
First(F) = { "(", "id" }
```

### Проверка LL(1)

```bash
# Запуск LL(1) анализа
make ll1-demo

# Вывод:
# First множества: { "E": {"(", "id"}, "E'": {"+", "ε"}, ... }
# Follow множества: { "E": {"$", ")"}, "E'": {"$", ")"}, ... }
# Грамматика является LL(1)
```

## Восстановление после ошибок

### Стратегии восстановления

1. **Панический режим** - пропуск до точки синхронизации
2. **Уровень фраз** - вставка/удаление токенов
3. **Продукции для ошибок** - специальные правила

### Метрики ошибок

```rust
ErrorMetrics {
    total_errors_detected: 50,       // Всего обнаружено
    actual_errors: 2,                // Уникальных ошибок
    cascading_prevented: 48,         // Предотвращено каскадных
    recovered_errors: 49,            // Успешно восстановлено
    recovery_quality: 0.98,          // Качество восстановления
}
```

### Примеры сообщений

```
program.src:10:5: ошибка: отсутствует ';' после return
  Совет: Добавьте ';' в конце инструкции

program.src:15:12: ошибка: неожиданная '}', найдено: '}'
  Совет: Проверьте, нет ли лишней закрывающей скобки
```

## Поддерживаемые конструкции языка

| Категория | Конструкции |
|-----------|-------------|
| **Функции** | Объявление, параметры, возвращаемые типы (`->`) |
| **Структуры** | Определение, поля, доступ (`.`), вложенность |
| **Переменные** | Объявление, инициализация, присваивание |
| **Управление** | `if-else`, `while`, `for`, `break` |
| **Инкременты** | `++x`, `x++`, `--x`, `x--` |
| **Выражения** | Арифметика, сравнения, логика |
| **Препроцессор** | `#define`, `#ifdef`, `#ifndef`, `#else`, `#endif` |

## Команда

- **Владимир (Feronski)** - Ведущий разработчик

## Полезные ссылки

- [Спецификация языка MiniC](docs/language_spec.md)
- [Формальная грамматика](docs/grammar.md)
- [LL(1) грамматики](https://ru.wikipedia.org/wiki/LL(1))
- [Примеры использования](examples/)
- [Чек-лист спринтов](docs/CHECKLIST.md)

**Версия:** 0.2.0
**Дата релиза:** Март 2026