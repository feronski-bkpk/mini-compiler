# Mini Compiler

**Минимальный компилятор C-подобного языка, написанный на Rust с полной поддержкой LL(1) грамматики, семантическим анализом и восстановлением после ошибок.**

## Оглавление

- [Особенности](#особенности)
- [Структура проекта](#структура-проекта)
- [Установка и сборка](#установка-и-сборка)
- [Makefile команды](#makefile-команды)
- [Быстрый старт](#быстрый-старт)
- [Использование CLI](#использование-cli)
- [Семантический анализ](#семантический-анализ)
- [Вывод типов (var)](#вывод-типов-var)
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
- **Приоритет операторов** - правильная обработка 10 уровней
- **Семантический анализатор** - проверка типов, областей видимости, вызовов функций
- **Вывод типов** - автоматическое определение типов через `var`
- **Таблица символов** - иерархическая с поддержкой вложенных областей
- **Размещение в памяти** - вычисление размеров и смещений для переменных и полей структур
- **Визуализация AST** - текстовый, DOT и JSON форматы
- **Визуализация таблицы символов** - вывод с размерами и смещениями
- **LL(1) анализ** - вычисление First/Follow множеств и проверка грамматики
- **Метрики ошибок** - качество восстановления, предотвращение каскадных ошибок
- **Обработка ошибок** - детальные сообщения на русском языке с предложениями по исправлению

### Технические характеристики
- **Язык**: Rust 2024 edition
- **Обработка ошибок**: Детальные сообщения с восстановлением
- **Поддержка кодировок**: UTF-8
- **Окончания строк**: Unix (`\n`) и Windows (`\r\n`)
- **Тесты**: 124+ тестов (unit + интеграционные + LL(1) + семантические)

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
│   ├── semantic/                 # Семантический анализ
│   │   ├── mod.rs                # Экспорт модуля
│   │   ├── analyzer.rs           # Основной анализатор
│   │   ├── symbol_table.rs       # Таблица символов
│   │   ├── type_system.rs        # Система типов
│   │   ├── errors.rs             # Семантические ошибки
│   │   └── pretty_printer.rs     # Вывод декорированного AST
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
│   ├── ll1_tests.rs              # 2 теста LL(1) анализа
│   └── semantic_tests.rs         # 24 теста семантического анализа
├── examples/                     # Демонстративные файлы
│   ├── hello.src                 # Простая программа
│   ├── factorial.src             # Рекурсивный факториал
│   ├── struct.src                # Работа со структурами
│   ├── increments.src            # Демонстрация инкрементов
│   ├── var_demo.src              # Демонстрация вывода типов
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
make test-semantic           # Семантические тесты
make test-common             # Тесты общих модулей
make test-doc                # Документационные тесты
make test-all                # Все тесты (включая LL(1) и семантику)
```

### Демонстрации
```bash
make ast-demo                # Визуализация AST
make inc-demo                # Демонстрация инкрементов
make error-demo              # Восстановление после ошибок
make ll1-demo                # LL(1) анализ грамматики
make semantic-demo           # Демонстрация семантического анализа
make var-demo                # Демонстрация вывода типов var
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
make semantic-demo
make var-demo
make full-pipeline
```

### 2. Полный пайплайн компиляции вручную

```bash
# Создайте тестовый файл с инкрементами и var
cat > examples/demo.src << 'EOF'
#define MAX 100
#define DEBUG 1

fn main() {
    int x = 5;
    x++;                    // Постфиксный инкремент
    ++x;                    // Префиксный инкремент
    var y = x++ + ++x;      // var с выводом типа
    
    #ifdef DEBUG
        var debug_var = y;
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

# Шаг 4: Семантический анализ
cargo run -- semantic --input examples/processed.src --show-symbols

# Шаг 5: Полный пайплайн одной командой
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

# Семантический анализ
cargo run -- semantic --input file.src
cargo run -- semantic --input file.src --show-symbols
cargo run -- semantic --input file.src --show-ast
cargo run -- semantic --input file.src --show-layout

# Препроцессор
cargo run -- preprocess --input file.src --output processed.src --show
cargo run -- preprocess --input file.src --defines "DEBUG=1" "VERSION=2"

# Полный пайплайн
cargo run -- full --input file.src --ast-format text
cargo run -- full --input file.src --show-metrics

# Проверка синтаксиса и семантики
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

## Семантический анализ

Семантический анализатор проверяет корректность программы на уровне типов и областей видимости.

### Команды семантического анализа

```bash
# Базовый семантический анализ
cargo run -- semantic --input program.src

# Вывод таблицы символов
cargo run -- semantic --input program.src --show-symbols

# Вывод с размерами и смещениями (SYM-5)
cargo run -- semantic --input program.src --show-symbols --show-layout

# Вывод декорированного AST
cargo run -- semantic --input program.src --show-ast
```

### Примеры семантических ошибок

**Необъявленная переменная:**
```c
fn main() -> int {
    int x = y + 5;  // y не объявлена
    return x;
}
```

Вывод:
```
семантическая ошибка: необъявленный идентификатор
  --> строка 2, столбец 13
  |
  | Переменная 'y' не объявлена
  |
  | совет: Объявите 'y' перед использованием
```

**Несоответствие типов:**
```c
fn main() -> int {
    int x = 3.14;  // float в int
    return x;
}
```

Вывод:
```
семантическая ошибка: несоответствие типов при присваивании
  --> строка 2, столбец 13
  |
  | ожидалось: int
  | получено: float
  |
  | совет: Используйте значение типа int или выполните явное приведение
```

**Ошибка вызова функции:**
```c
fn add(int a, int b) -> int {
    return a + b;
}

fn main() -> int {
    return add(5);  // не хватает аргумента
}
```

Вывод:
```
семантическая ошибка: несоответствие количества аргументов
  --> строка 6, столбец 12
  |
  | Функция 'add' ожидает 2 аргументов, получено 1
  |
  | совет: Функция объявлена как add (int, int) -> int
```

## Вывод типов (var)

Ключевое слово `var` позволяет компилятору автоматически определить тип переменной из инициализатора.

### Синтаксис
```c
var x = 42;     // выводится int
var y = 3.14;   // выводится float
var z = true;   // выводится bool
var s = "hello"; // выводится string
```

### Правила использования

1. **Обязательный инициализатор:**
   ```c
   var x;        // Ошибка: var требует инициализатора
   ```

2. **Тип фиксируется после вывода:**
   ```c
   var x = 42;   // x: int
   x = 100;      // OK
   x = 3.14;     // Ошибка: int != float
   ```

3. **Поддержка в циклах:**
   ```c
   for (var i = 0; i < 10; i = i + 1) {
       // i: int
   }
   ```

### Пример с var
```bash
# Создайте файл var_demo.src
cat > var_demo.src << 'EOF'
fn main() -> int {
    var x = 42;
    var y = 3.14;
    var z = true;
    var s = "hello";
    
    x = 100;
    y = 2.71;
    
    return 0;
}
EOF

# Запустите семантический анализ
cargo run -- semantic --input var_demo.src --show-symbols
```

Вывод таблицы символов покажет выведенные типы.

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

### Демонстрация семантического анализа
```bash
make semantic-demo
# или
cargo run -- semantic --input examples/errors.src --show-symbols
```

Показывает:
- Проверку типов
- Обнаружение необъявленных переменных
- Проверку вызовов функций
- Детальные сообщения об ошибках на русском языке

### Демонстрация вывода типов (var)
```bash
make var-demo
# или
cargo run -- semantic --input examples/var_demo.src --show-symbols --show-layout
```

Показывает:
- Автоматический вывод типов
- Размеры и смещения переменных в памяти
- Проверку совместимости при присваивании

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
    Block [line 1]:
      VarDecl: int x = 5
      Expr: (x++)
      Expr: (++x)
      VarDecl: int y = (x++) + (++z)
      Return: y
```

### Декорированное AST с типами
```bash
cargo run -- semantic --input examples/factorial.src --show-ast
```

Пример вывода:
```
Program [global scope]:
  Symbol Table:
    Global:
      factorial: function(int) -> int
      main: function() -> int

  FunctionDecl: factorial -> int [line 1]:
    Parameters:
      - n: int
    Body [type checked]:
      Block [line 2]:
        IfStmt:
          Condition: (n <= 1) [type: bool]
          Then branch:
            Return: 1 [type: int]
        Return: (n * factorial(n - 1)) [type: int]
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

# Конкретные модули (через Makefile)
make test-lexer
make test-parser
make test-semantic
make test-preprocessor
make test-ll1
make test-integration
make test-common
make test-all
```

### Примеры тестов

**Семантический тест (типы):**
```rust
#[test]
fn test_type_mismatch() {
    let source = "fn main() { int x = 3.14; }";
    let (valid, errors) = analyze(source);
    assert!(!valid);
    assert!(errors.contains(&SemanticErrorKind::AssignmentTypeMismatch));
}
```

**Семантический тест (var):**
```rust
#[test]
fn test_var_type_inference() {
    let source = r#"
        fn main() {
            var x = 42;
            var y = 3.14;
            var z = true;
            var s = "hello";
        }
    "#;
    let (valid, errors) = analyze(source);
    assert!(valid);
}
```

**Тест восстановления после ошибок:**
```rust
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
   - **Вывод типов (var)**
   - **Семантические правила**
   - Примеры программ

2. **Формальная грамматика** - `docs/grammar.md`
   - Детальное описание всех правил
   - Приоритет операторов (10 уровней)
   - EBNF нотация
   - LL(1) свойства
   - **Тип var**

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
          ▼
   ┌──────────────┐
   │   Семантика  │  ← Проверка типов, таблица символов
   └──────────────┘
          │
          ├──────────▶ Декорированное AST
          │
          ├──────────▶ Таблица символов
          │
          ├──────────▶ Семантические ошибки
          │
          └──────────▶ Вывод типов (var)
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
- Поддержка `var` как типа

#### Семантический анализатор (`src/semantic/`)
- Иерархическая таблица символов
- Полная система типов
- Проверка объявлений и областей видимости
- Проверка типов выражений
- Проверка вызовов функций
- Проверка структур и доступа к полям
- Вывод типов для `var`
- Вычисление размеров и смещений в памяти
- Детальные сообщения об ошибках на русском языке

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

**Синтаксические ошибки:**
```
program.src:10:5: ошибка: отсутствует ';' после return
  Совет: Добавьте ';' в конце инструкции

program.src:15:12: ошибка: неожиданная '}', найдено: '}'
  Совет: Проверьте, нет ли лишней закрывающей скобки
```

**Семантические ошибки (на русском):**
```
семантическая ошибка: необъявленный идентификатор
  --> строка 2, столбец 18
  |
  | Переменная 'value' не объявлена
  |
  | совет: Объявите 'value' перед использованием
```

## Поддерживаемые конструкции языка

| Категория | Конструкции |
|-----------|-------------|
| **Функции** | Объявление, параметры, возвращаемые типы (`->`) |
| **Структуры** | Определение, поля, доступ (`.`), вложенность |
| **Переменные** | Объявление, инициализация, присваивание, **вывод типов (`var`)** |
| **Управление** | `if-else`, `while`, `for`, `break` |
| **Инкременты** | `++x`, `x++`, `--x`, `x--` |
| **Выражения** | Арифметика, сравнения, логика |
| **Препроцессор** | `#define`, `#ifdef`, `#ifndef`, `#else`, `#endif` |
| **Типы** | `int`, `float`, `bool`, `void`, `string`, `struct`, **`var`** |

## Команда

- **Владимир (Feronski)** - Ведущий разработчик

## Полезные ссылки

- [Спецификация языка MiniC](docs/language_spec.md)
- [Формальная грамматика](docs/grammar.md)
- [Примеры использования](examples/)
- [Чек-лист спринтов](docs/CHECKLIST.md)

**Версия:** 0.3.0
**Дата релиза:** Март 2026