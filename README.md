# Mini Compiler

**Минимальный компилятор C-подобного языка, написанный на Rust.**

[![Rust](https://img.shields.io/badge/rust-2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Tests](https://img.shields.io/badge/tests-79%20-blue)]()
[![Sprint](https://img.shields.io/badge/sprint-2%20complete-purple)]()

## Оглавление

- [Особенности](#особенности)
- [Структура проекта](#структура-проекта)
- [Установка и сборка](#установка-и-сборка)
- [Быстрый старт](#быстрый-старт)
- [Использование](#использование)
- [Визуализация AST](#визуализация-ast)
- [Тестирование](#тестирование)
- [Документация](#документация)
- [Архитектура](#архитектура)
- [Команда](#команда)
- [Полезные ссылки](#полезные-ссылки)

## Особенности

### Реализовано
- **Лексический анализатор** - полное распознавание всех типов токенов
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
- **Приоритет операторов** - правильная обработка 9 уровней
- **Визуализация AST** - текстовый, DOT и JSON форматы
- **Обработка ошибок** - детальные сообщения с восстановлением

### Технические характеристики
- **Язык**: Rust 2024 edition
- **Обработка ошибок**: Детальные сообщения с восстановлением
- **Поддержка кодировок**: UTF-8
- **Окончания строк**: Unix (`\n`) и Windows (`\r\n`)
- **Тесты**: 79+ тестов (unit + интеграционные)

## Структура проекта

```
mini-compiler/
├── src/                      # Исходный код
│   ├── common/               # Общие типы данных
│   │   ├── mod.rs            # Утилиты
│   │   ├── token.rs          # Токены (30+ типов)
│   │   └── position.rs       # Позиция в исходном коде
│   ├── lexer/                # Лексический анализатор
│   │   ├── mod.rs            # Основной модуль
│   │   ├── scanner.rs        # Сканер (основная логика)
│   │   └── error.rs          # Ошибки лексического анализа
│   ├── parser/               # Парсер
│   │   ├── mod.rs            # Экспорт модуля
│   │   ├── parser.rs         # Рекурсивный спуск
│   │   ├── ast.rs            # Структуры AST
│   │   ├── error.rs          # Ошибки парсера
│   │   ├── visitor.rs        # Паттерн Visitor
│   │   ├── pretty_printer.rs # Текстовый вывод AST
│   │   ├── dot_generator.rs  # Graphviz DOT генератор
│   │   ├── json_generator.rs # JSON генератор
│   │   └── grammar.txt       # Формальная грамматика
│   ├── preprocessor/         # Препроцессор
│   │   ├── mod.rs            # Основной модуль
│   │   ├── macros.rs         # Таблица макросов
│   │   └── error.rs          # Ошибки препроцессора
│   ├── utils/                # Вспомогательные функции
│   ├── lib.rs                # Точка входа библиотеки
│   └── main.rs               # Точка входа CLI
├── tests/                    # Тестовые файлы
│   ├── lexer/                # Тесты лексера
│   ├── parser/               # Тесты парсера
│   │   ├── valid/            # Корректные программы
│   │   ├── invalid/          # Некорректные программы
│   │   └── ast_output/       # Ожидаемые результаты
│   ├── lexer_tests.rs        # 7 тестов лексера
│   ├── parser_tests.rs       # 25 тестов парсера
│   ├── preprocessor_tests.rs # 8 тестов препроцессора
│   └── integration_tests.rs  # 2 интеграционных теста
├── examples/                 # Демонстративные файлы
│   ├── hello.src             # Простая программа
│   ├── factorial.src         # Рекурсивный факториал
│   └── struct.src            # Работа со структурами
├── docs/                     # Документация
│   ├── CHECKLIST.md          # Чек-лист по спринтам
│   ├── language_spec.md      # Спецификация языка
│   └── grammar.md            # Формальная грамматика
├── Cargo.toml                # Конфигурация Cargo
├── Cargo.lock                # Версии зависимостей
├── Makefile                  # Система сборки
├── README.md                 # Этот файл
└── .gitignore                # Игнорируемые файлы Git
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
cargo build

# Или собрать в режиме релиза
cargo build --release
```

### Доступные команды сборки

```bash
# Проверка компиляции
cargo check

# Форматирование кода
cargo fmt

# Проверка стиля кода
cargo clippy

# Сборка и запуск с Make
make build
make test
make run
```

## Быстрый старт

### 1. Демонстрация всех возможностей

```bash
# Запуск демонстрационной программы с AST
make ast-demo
```

### 2. Полный пайплайн компиляции

```bash
# Создайте тестовый файл
cat > test.src << 'EOF'
#define MAX 100
#define GREETING "Hello, World!"

#ifdef DEBUG
    log("Debug mode enabled");
#endif

fn main() {
    int x = MAX;
    string msg = GREETING;
    
    // Комментарий
    if (x > 0) {
        return x * 2;
    }
    
    return 0;
}
EOF

# 1. Препроцессор
cargo run -- preprocess --input test.src --defines "DEBUG=1" --show

# 2. Лексический анализ
cargo run -- lex --input test.src --verbose

# 3. Синтаксический анализ с построением AST
cargo run -- parse --input test.src --ast-format text

# 4. Полный пайплайн
cargo run -- full --input test.src --defines "DEBUG=1" --ast-format dot --output ast.dot
```

## Использование

### CLI Интерфейс

```bash
# Показать справку
cargo run -- --help

# Показать информацию о компиляторе
cargo run -- info
cargo run -- info --verbose

# Показать спецификацию языка
cargo run -- spec
```

### Основные команды

#### Лексический анализ
```bash
# Анализ файла
cargo run -- lex --input program.src

# Интерактивный режим
cargo run -- lex --interactive

# С выводом в файл
cargo run -- lex --input program.src --output tokens.txt

# JSON формат
cargo run -- lex --input program.src --format json
```

#### Синтаксический анализ
```bash
# Базовый анализ с выводом AST
cargo run -- parse --input program.src

# Текстовый формат AST
cargo run -- parse --input program.src --ast-format text

# Graphviz DOT формат
cargo run -- parse --input program.src --ast-format dot --output ast.dot

# JSON формат
cargo run -- parse --input program.src --ast-format json --output ast.json

# С препроцессором
cargo run -- parse --input program.src --preprocess --defines "DEBUG=1"
```

#### Препроцессор
```bash
# Обработка препроцессором
cargo run -- preprocess --input program.src --output processed.src

# С макросами
cargo run -- preprocess --input program.src --defines "DEBUG=1" "VERSION=2"

# Сохранять нумерацию строк
cargo run -- preprocess --input program.src --preserve-lines
```

#### Полный пайплайн
```bash
# Препроцессор + лексический анализ + синтаксический анализ
cargo run -- full --input program.src --defines "FEATURE=1"

# С сохранением AST в DOT формате
cargo run -- full --input program.src --ast-format dot --output program.dot
```

#### Проверка синтаксиса
```bash
# Проверка файла
cargo run -- check --input program.src

# Строгая проверка
cargo run -- check --input program.src --strict

# С препроцессором
cargo run -- check --input program.src --preprocess --defines "DEBUG=1"
```

## Визуализация AST

### Текстовый формат
```bash
cargo run -- parse --input examples/factorial.src
```

Пример вывода:
```
Program [line 1]:
  FunctionDecl: factorial -> int [line 1]:
    Parameters: [n: int]
    Body [line 1]:
      Block [line 2-5]:
        IfStmt [line 2]:
          Condition: (n <= 1)
          Then:
            Block [line 3]:
              Return: 1
        Return: (n * factorial((n - 1)))
  
  FunctionDecl: main -> void [line 7]:
    Parameters: []
    Body [line 7]:
      Block [line 8-9]:
        VarDecl: int result = factorial(5)
        Return: result
```

### Graphviz DOT формат
```bash
# Генерация DOT файла
cargo run -- parse --input examples/struct.src --ast-format dot --output ast.dot

# Визуализация (требуется Graphviz)
dot -Tpng ast.dot -o ast.png
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

# Конкретные модули (Makefile)
make test-lexer        # Тесты лексера
make test-parser       # Тесты парсера
make test-preprocessor # Тесты препроцессора
make test-integration  # Интеграционные тесты
make test-common       # Тесты общих модулей
```

### Статистика тестов

| Категория | Количество тестов |
|-----------|-------------------|
| Unit-тесты (common, lexer) | 30 |
| Тесты парсера | 25 |
| Тесты препроцессора | 8 |
| Интеграционные тесты | 2 |
| **Всего** | **65+** |

### Примеры тестов

```rust
// Тест парсера для функции с параметрами
#[test]
fn test_function_with_params() {
    assert_parses("fn add(int a, int b) -> int { return a + b; }");
}

// Тест на синтаксическую ошибку
#[test]
fn test_missing_semicolon() {
    assert_parse_error("fn main() { return 42 }");
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
   - Примеры программ

2. **Формальная грамматика** - `docs/grammar.md`
   - Детальное описание всех правил
   - Приоритет операторов
   - EBNF нотация

3. **API документация** - автоматически генерируется
   - Все публичные функции и структуры
   - Примеры использования
   - Типы ошибок

## Архитектура

### Текущая реализация

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
   │   Парсер     │  ← Синтаксический анализ
   └──────────────┘
          │
          ├──────────▶ AST (Abstract Syntax Tree)
          │
          ├──────────▶ Визуализация (text/dot/json)
          │
          └──────────▶ Сообщения об ошибках
```

### Компоненты системы

#### 1. **Препроцессор** (`src/preprocessor/`)
- Удаление комментариев (`//` и `/* */`)
- Подстановка макросов (`#define`)
- Условная компиляция (`#ifdef`, `#ifndef`, `#endif`, `#else`)
- Обнаружение рекурсии макросов
- Сохранение нумерации строк

#### 2. **Лексический анализатор** (`src/lexer/`)
- **Сканер** - преобразование текста в токены
- **Токены** - 30+ типов
- **Позиционирование** - точное отслеживание строк и колонок
- **Обработка ошибок** - восстановление после недопустимых символов

#### 3. **Парсер** (`src/parser/`)
- **Рекурсивный спуск** - LL(1) грамматика
- **AST** - полная иерархия узлов
- **Приоритет операторов** - 9 уровней
- **Visitor pattern** - для обхода и анализа
- **Визуализация** - text, dot, json форматы

#### 4. **Общие типы** (`src/common/`)
- `Token` - токен с типом, лексемой и позицией
- `Position` - позиция в исходном коде
- `TokenKind` - перечисление всех типов токенов

#### 5. **CLI интерфейс** (`src/main.rs`)
- 10+ команд: `lex`, `parse`, `check`, `preprocess`, `full`, `test`, `docs`, `info`, `spec`
- 4 формата вывода: `text`, `json`, `minimal`, `verbose`
- 3 формата AST: `text`, `dot`, `json`

### Поддерживаемые конструкции языка

| Категория | Конструкции |
|-----------|-------------|
| **Функции** | Объявление, параметры, возвращаемые типы (`->`) |
| **Структуры** | Определение, поля, доступ (`.`), вложенность |
| **Переменные** | Объявление, инициализация, присваивание |
| **Управление** | `if-else`, `while`, `for`, `break` |
| **Выражения** | Арифметика, сравнения, логика, приоритеты |
| **Вызовы** | Функции с аргументами, вложенные вызовы |
| **Литералы** | Целые, float, строки, булевы |
| **Препроцессор** | `#define`, `#ifdef`, `#ifndef`, `#else`, `#endif` |

## Команда

- **Владимир (Feronski)** - Ведущий разработчик

### Как присоединиться?
1. Форкните репозиторий
2. Изучите открытые Issues
3. Создайте ветку для своей задачи
4. Реализуйте изменения с тестами
5. Отправьте Pull Request

## Полезные ссылки

- [Спецификация языка MiniC](docs/language_spec.md) - полное описание языка
- [Формальная грамматика](docs/grammar.md) - EBNF спецификация
- [Примеры использования](examples/) - демонстрационные программы
- [Тестовые примеры](tests/) - все категории тестов
- [Чек-лист спринтов](docs/CHECKLIST.md) - прогресс разработки

**Версия:** 0.2.0
**Дата релиза:** Март 2026