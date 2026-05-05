# Mini Compiler

**Минимальный компилятор C-подобного языка, написанный на Rust с полной поддержкой LL(1) грамматики, семантическим анализом, генерацией промежуточного представления (IR) и генерацией x86-64 ассемблерного кода.**

## Оглавление

- [Особенности](#особенности)
- [Структура проекта](#структура-проекта)
- [Установка и сборка](#установка-и-сборка)
- [Makefile команды](#makefile-команды)
- [Быстрый старт](#быстрый-старт)
- [Использование CLI](#использование-cli)
- [Генерация промежуточного представления (IR)](#генерация-промежуточного-представления-ir)
- [Генерация x86-64 ассемблерного кода](#генерация-x86-64-ассемблерного-кода)
- [Управляющие конструкции](#управляющие-конструкции-sprint-6)
- [Короткая схема вычислений](#короткая-схема-вычислений)
- [Семантический анализ](#семантический-анализ)
- [Вывод типов (var)](#вывод-типов-var)
- [Оптимизации IR](#оптимизации-ir)
- [Демонстрации](#демонстрации)
- [Визуализация AST и CFG](#визуализация-ast-и-cfg)
- [Тестирование](#тестирование)
- [Документация](#документация)
- [Архитектура](#архитектура)
- [LL(1) анализ](#ll1-анализ)
- [Восстановление после ошибок](#восстановление-после-ошибок)
- [Команда](#команда)
- [Полезные ссылки](#полезные-ссылки)

## Особенности

### Технические характеристики
- **Язык**: Rust 2024 edition
- **Целевая архитектура**: x86-64
- **ABI**: System V AMD64
- **Ассемблер**: NASM
- **Обработка ошибок**: Детальные сообщения с восстановлением
- **Поддержка кодировок**: UTF-8
- **Окончания строк**: Unix (`\n`) и Windows (`\r\n`)

### Новые возможности Sprint 6

| Возможность               | Описание                                                    |
|---------------------------|-------------------------------------------------------------|
| **Условные операторы**    | `if`, `if-else`, вложенные условия с правильными метками    |
| **Циклы**                 | `while`, `for` с оптимизацией счетных циклов                |
| **Switch**                | Оператор выбора `switch` с `case` и `default`               |
| **Break/Continue**        | Выход из цикла и переход к следующей итерации               |
| **Короткая схема**        | `&&` и `\|\|` не вычисляют правый операнд при необходимости |
| **Float операции**        | Сравнения через `ucomisd`, приведение `int→float`           |
| **Беззнаковые сравнения** | `setb`/`seta` для беззнаковых операций                      |
| **Массивы**               | Статические массивы фиксированного размера                  |

## Структура проекта

```
mini-compiler/
├── src/                              # Исходный код
│   ├── common/                       # Общие типы данных
│   │   ├── mod.rs                    # Утилиты
│   │   ├── token.rs                  # Токены (18 ключевых слов)
│   │   └── position.rs               # Позиция в исходном коде
│   ├── lexer/                        # Лексический анализатор
│   │   ├── mod.rs                    # Основной модуль
│   │   ├── scanner.rs                # Сканер (основная логика)
│   │   └── error.rs                  # Ошибки лексического анализа
│   ├── parser/                       # Парсер
│   │   ├── mod.rs                    # Экспорт модуля
│   │   ├── parser.rs                 # Рекурсивный спуск
│   │   ├── ast.rs                    # Структуры AST (включая Switch, Array)
│   │   ├── error.rs                  # Ошибки парсера с метриками
│   │   ├── visitor.rs                # Паттерн Visitor
│   │   ├── pretty_printer.rs         # Текстовый вывод AST
│   │   ├── dot_generator.rs          # Graphviz DOT генератор
│   │   ├── json_generator.rs         # JSON генератор
│   │   ├── ll1.rs                    # LL(1) анализ (First/Follow)
│   │   ├── error_productions.rs      # Продукции для ошибок
│   │   └── grammar.txt               # Формальная грамматика
│   ├── semantic/                     # Семантический анализ
│   │   ├── mod.rs                    # Экспорт модуля
│   │   ├── analyzer.rs               # Основной анализатор
│   │   ├── symbol_table.rs           # Таблица символов
│   │   ├── type_system.rs            # Система типов
│   │   ├── errors.rs                 # Семантические ошибки
│   │   └── pretty_printer.rs         # Вывод декорированного AST
│   ├── ir/                           # Промежуточное представление
│   │   ├── mod.rs                    # Экспорт модуля
│   │   ├── ir_instructions.rs        # Определения инструкций IR
│   │   ├── ir_generator.rs           # Генератор IR из AST
│   │   ├── basic_block.rs            # Базовые блоки и CFG
│   │   ├── control_flow.rs           # Построение CFG
│   │   ├── ir_printer.rs             # Вывод IR (текст, DOT, JSON)
│   │   └── peephole_optimizer.rs     # Оптимизации IR
│   ├── codegen/                      # Генерация x86-64 кода
│   │   ├── mod.rs                    # Экспорт модуля
│   │   ├── x86_generator.rs          # Генератор x86-64 ассемблера
│   │   ├── control_flow_generator.rs # Генератор управляющих конструкций
│   │   ├── expression_generator.rs   # Генератор выражений
│   │   ├── label_manager.rs          # Менеджер меток
│   │   ├── stack_frame.rs            # Управление стековым фреймом
│   │   ├── register_allocator.rs     # Аллокатор регистров
│   │   └── abi.rs                    # System V AMD64 ABI константы
│   ├── runtime/                      # Рантайм-библиотека
│   │   └── runtime.asm               # Ассемблерные функции (print_int, exit, etc.)
│   ├── preprocessor/                 # Препроцессор
│   │   ├── mod.rs                    # Основной модуль
│   │   ├── macros.rs                 # Таблица макросов
│   │   └── error.rs                  # Ошибки препроцессора
│   ├── utils/                        # Вспомогательные функции
│   ├── lib.rs                        # Точка входа библиотеки
│   └── main.rs                       # Точка входа CLI
├── tests/                            # Тестовые файлы
│   ├── control_flow/                 # Тесты потока управления (Sprint 6)
│   │   ├── valid/
│   │   │   ├── conditionals/         # Тесты условных операторов
│   │   │   ├── loops/                # Тесты циклов
│   │   │   ├── logical_ops/          # Тесты логических операторов
│   │   │   └── complex_expressions/  # Тесты сложных выражений
│   │   └── invalid/
│   │       ├── infinite_loops/       # Бесконечные циклы
│   │       └── type_errors/          # Ошибки типов
│   ├── control_flow_tests.rs         # Тесты потока управления
│   ├── lexer/                        # Тесты лексера
│   ├── parser/                       # Тесты парсера
│   ├── ir/                           # Тесты IR
│   ├── codegen/                      # Тесты кодогенерации
│   ├── lexer_tests.rs                # Тесты лексера
│   ├── parser_tests.rs               # Тесты парсера
│   ├── preprocessor_tests.rs         # Тесты препроцессора
│   ├── integration_tests.rs          # Интеграционные тесты
│   ├── ll1_tests.rs                  # Тесты LL(1) анализа
│   ├── semantic_tests.rs             # Тесты семантического анализа
│   ├── ir_tests.rs                   # Тесты IR генерации
│   ├── ir_optimization_tests.rs      # Тесты оптимизаций
│   ├── ir_golden_tests.rs            # Golden тесты IR
│   ├── codegen_tests.rs              # Тесты кодогенерации
│   ├── integration_codegen.rs        # Интеграционные тесты кодогенерации
│   ├── abi_compliance_tests.rs       # ABI compliance тесты
│   └── bugs_tests.rs                 # Тесты исправленных ошибок
├── examples/                         # Демонстрационные файлы
├── docs/                             # Документация
│   ├── CHECKLIST.md                  # Чек-лист по спринтам
│   ├── language_spec.md              # Спецификация языка
│   └── grammar.md                    # Формальная грамматика
├── Cargo.toml                        # Конфигурация Cargo
├── Cargo.lock                        # Версии зависимостей
├── Makefile                          # Система сборки
├── README.md                         # Этот файл
└── .gitignore                        # Игнорируемые файлы Git
```

## Установка и сборка

### Предварительные требования

- **Rust 1.70 или новее**
- **Cargo** (менеджер пакетов Rust)
- **Git** (для клонирования репозитория)
- **NASM** (для ассемблирования сгенерированного кода)
- **Graphviz** (опционально, для визуализации AST и CFG)

### Установка NASM

**Windows (MinGW):**
```bash
# Через MSYS2
pacman -S mingw-w64-x86_64-nasm

# Или скачайте с https://www.nasm.us/
```

**Linux:**
```bash
sudo apt-get install nasm
```

**macOS:**
```bash
brew install nasm
```

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
make test-semantic           # Семантические тесты
make test-ir                 # Тесты IR генерации
make test-ir-opt             # Тесты оптимизаций
make test-codegen            # Тесты кодогенерации
make test-control-flow       # Тесты потока управления
make test-abi                # ABI compliance тесты
make test-all                # Все тесты
```

### Демонстрации Sprint 6 (НОВЫЕ!)
```bash
make sprint6-demo            # Полная демонстрация всех новых возможностей
make switch-demo             # Демонстрация Switch-case-default
make break-continue-demo     # Демонстрация Break и Continue
make short-circuit-demo      # Демонстрация короткой схемы вычислений
make float-demo              # Демонстрация Float и приведения типов
make array-demo              # Демонстрация массивов
```

### Демонстрации
```bash
make ast-demo                # Визуализация AST
make ir-demo                 # Демонстрация IR генерации
make codegen-demo            # Демонстрация кодогенерации
make optimization-demo       # Демонстрация оптимизаций
make semantic-demo           # Демонстрация семантического анализа
make var-demo                # Демонстрация вывода типов var
make inc-demo                # Демонстрация инкрементов
make error-demo              # Восстановление после ошибок
make ll1-demo                # LL(1) анализ грамматики
make full-pipeline           # Полный пайплайн
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

### Примеры использования
```bash
make example                 # Примеры использования компилятора
```

## Быстрый старт

### 1. Быстрый запуск всех демонстраций Sprint 6

```bash
# Собрать проект
make build

# Запустить демонстрацию Sprint 6
make sprint6-demo

# Или отдельные демонстрации
make switch-demo
make break-continue-demo
make short-circuit-demo
make float-demo
make array-demo
```

### 2. Полный пайплайн компиляции вручную

```bash
# Создайте тестовый файл
cat > test.src << 'EOF'
fn factorial(int n) -> int {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn main() -> int {
    return factorial(5);
}
EOF

# Шаг 1: Препроцессор
cargo run -- preprocess --input test.src --output processed.src --show

# Шаг 2: Лексический анализ
cargo run -- lex --input processed.src --verbose

# Шаг 3: Синтаксический анализ
cargo run -- parse --input processed.src --ast-format text

# Шаг 4: Семантический анализ
cargo run -- semantic --input processed.src --show-symbols

# Шаг 5: Генерация IR
cargo run -- ir --input processed.src --ir-format text

# Шаг 6: Генерация x86-64 ассемблера
cargo run -- codegen --input processed.src --output factorial.asm

# Шаг 7: Сборка и запуск
nasm -f elf64 factorial.asm -o factorial.o
gcc -no-pie -o factorial factorial.o
./factorial
echo $?  # Должно вывести 120

# Шаг 8: Полный пайплайн одной командой
cargo run -- full --input test.src --ast-format dot --output ast.dot --show-metrics
```

## Использование CLI

### Команды CLI

```bash
# Информация
cargo run -- info                    # Базовая информация
cargo run -- info --verbose          # Подробная информация о всех возможностях
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

# Генерация промежуточного представления (IR)
cargo run -- ir --input file.src --ir-format text
cargo run -- ir --input file.src --ir-format dot --output cfg.dot
cargo run -- ir --input file.src --ir-format json --output ir.json
cargo run -- ir --input file.src --stats
cargo run -- ir --input file.src --optimize --verbose

# Генерация x86-64 ассемблерного кода
cargo run -- codegen --input file.src --output output.asm
cargo run -- codegen --input file.src --output output.asm --optimize
cargo run -- codegen --input file.src --output output.asm --stats

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

# Пример Sprint 6
cargo run --example sprint6_demo

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

# Форматы для IR
--ir-format text   # Текстовый IR (по умолчанию)
--ir-format dot    # Graphviz DOT для визуализации CFG
--ir-format json   # JSON для машинной обработки
```

## Генерация x86-64 ассемблерного кода

### Обзор

Генератор кода преобразует IR инструкции в валидный x86-64 ассемблер с соблюдением **System V AMD64 ABI**.

### Поддерживаемые инструкции

| Категория              | Инструкции                                                     |
|------------------------|----------------------------------------------------------------|
| **Арифметические**     | `ADD → add`, `SUB → sub`, `MUL → imul`, `DIV → idiv`           |
| **Логические**         | `AND → and`, `OR → or`, `NOT → not`, `XOR → xor`               |
| **Сравнения**          | `CMP_* → cmp + set*`                                           |
| **Float сравнения**    | `CMP_*F → ucomisd + set*`                                      |
| **Беззнаковые**        | `CMP_*U → setb/seta`                                           |
| **Память**             | `LOAD → mov reg, [mem]`, `STORE → mov [mem], reg`              |
| **Массивы**            | `ArrayLoad`, `ArrayStore`                                      |
| **Преобразование**     | `IntToFloat → cvtsi2sd`, `FloatToInt → cvttsd2si`              |
| **Управление потоком** | `JUMP → jmp`, `JUMP_IF → jnz`, `CALL → call`, `RETURN → ret`   |

### System V AMD64 ABI

**Соглашение о вызовах:**
- Первые 6 целочисленных аргументов: `RDI`, `RSI`, `RDX`, `RCX`, `R8`, `R9`
- Первые 8 аргументов с плавающей точкой: `XMM0-XMM7`
- Дополнительные аргументы: в стеке (справа налево)
- Возвращаемое значение: `RAX` (целые), `XMM0` (float)

**Использование регистров:**
- **Caller-saved**: `RAX`, `RCX`, `RDX`, `RSI`, `RDI`, `R8-R11`
- **Callee-saved**: `RBX`, `RSP`, `RBP`, `R12-R15`

### Примеры генерации

**Простая функция:**
```c
// Исходный код
fn main() -> int {
    return 42;
}
```

```asm
; Сгенерированный ассемблер
section .text
global main
global _start

main:
    push rbp
    mov rbp, rsp
    mov rax, 42
    mov rsp, rbp
    pop rbp
    ret

_start:
    call main
    mov rdi, rax
    call exit

exit:
    mov rax, 60
    syscall
```

**Функция с параметрами:**
```c
// Исходный код
fn add(int a, int b) -> int {
    return a + b;
}

fn main() -> int {
    return add(5, 3);
}
```

```asm
; Сгенерированный ассемблер (фрагмент)
main:
    push rbp
    mov rbp, rsp
    mov rdi, 5
    mov rsi, 3
    call add
    mov rsp, rbp
    pop rbp
    ret

add:
    push rbp
    mov rbp, rsp
    mov [rbp+16], rdi
    mov [rbp+24], rsi
    mov rax, [rbp+16]
    add rax, [rbp+24]
    mov rsp, rbp
    pop rbp
    ret
```

**If-Else:**
```c
// Исходный код
if (x > 0) {
    y = 10;
} else {
    y = 20;
}
```

```asm
; Сгенерированный ассемблер
    mov eax, [rbp-8]      ; Загрузить x
    cmp eax, 0
    jle .Lelse            ; Переход в else если x <= 0
    mov dword [rbp-12], 10 ; y = 10
    jmp .Lendif
.Lelse:
    mov dword [rbp-12], 20 ; y = 20
.Lendif:
```

**While цикл:**
```c
// Исходный код
while (i < 10) {
    sum = sum + i;
    i = i + 1;
}
```

```asm
; Сгенерированный ассемблер
.Lwhile_cond:
    mov eax, [rbp-4]       ; Загрузить i
    cmp eax, 10
    jge .Lwhile_end        ; Выход если i >= 10
    mov eax, [rbp-8]       ; Загрузить sum
    add eax, [rbp-4]       ; Добавить i
    mov [rbp-8], eax       ; Сохранить sum
    mov eax, [rbp-4]
    add eax, 1
    mov [rbp-4], eax
    jmp .Lwhile_cond
.Lwhile_end:
```

**Короткая схема (Short-Circuit):**
```c
// Исходный код
if (a != 0 && b / a > 2) {
    result = 1;
}
```

```asm
; Сгенерированный ассемблер
    mov eax, [rbp-8]       ; Загрузить a
    cmp eax, 0
    je .Lfalse             ; Если a == 0, короткая схема -> false
    mov eax, [rbp-12]      ; Загрузить b
    cdq
    idiv dword [rbp-8]     ; Разделить на a (только если a != 0)
    cmp eax, 2
    jg .Ltrue
    jmp .Lfalse
.Ltrue:
    mov eax, 1
    jmp .Lend
.Lfalse:
    mov eax, 0
.Lend:
```

**Switch:**
```c
// Исходный код
switch (x) {
    case 1: result = 10;
    case 2: result = 20;
    default: result = 0;
}
```

```asm
; Сгенерированный ассемблер
    cmp eax, 1
    sete al
    movzx rax, al
    cmp rax, 0
    jne .L_case1
    cmp eax, 2
    sete al
    movzx rax, al
    cmp rax, 0
    jne .L_case2
    jmp .L_default
.L_case1:
    mov dword [rbp-X], 10
    jmp .L_switch_end
.L_case2:
    mov dword [rbp-X], 20
    jmp .L_switch_end
.L_default:
    mov dword [rbp-X], 0
.L_switch_end:
```

### Команды кодогенерации

```bash
# Базовая генерация
cargo run -- codegen --input factorial.src --output factorial.asm

# С оптимизациями
cargo run -- codegen --input factorial.src --output factorial.asm --optimize

# Со статистикой
cargo run -- codegen --input factorial.src --output factorial.asm --stats

# Сборка и запуск
nasm -f elf64 factorial.asm -o factorial.o
gcc -no-pie -o factorial factorial.o
./factorial
```

### Стековый фрейм

**Пролог функции:**
```asm
push rbp        ; Сохранение базового указателя
mov rbp, rsp    ; Установка нового базового указателя
sub rsp, N      ; Выделение места для локальных переменных (выровнено по 16)
```

**Эпилог функции:**
```asm
mov rsp, rbp    ; Восстановление указателя стека
pop rbp         ; Восстановление базового указателя
ret             ; Возврат к вызывающей функции
```

## Генерация промежуточного представления (IR)

### Набор инструкций IR

IR использует формат трехадресного кода с поддержкой:

| Категория | Инструкции |
|-----------|------------|
| **Арифметические** | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `NEG` |
| **Логические** | `AND`, `OR`, `NOT`, `XOR` |
| **Сравнения** | `CMP_EQ`, `CMP_NE`, `CMP_LT`, `CMP_LE`, `CMP_GT`, `CMP_GE` |
| **Float сравнения** | `CMP_EQF`, `CMP_NEF`, `CMP_LTF`, `CMP_LEF`, `CMP_GTF`, `CMP_GEF` |
| **Беззнаковые сравнения** | `CMP_LTU`, `CMP_LEU`, `CMP_GTU`, `CMP_GEU` |
| **Память** | `LOAD`, `STORE`, `ALLOCA`, `GEP` |
| **Массивы** | `ArrayLoad`, `ArrayStore` |
| **Преобразование** | `IntToFloat`, `FloatToInt` |
| **Управление потоком** | `JUMP`, `JUMP_IF`, `JUMP_IF_NOT`, `LABEL`, `PHI` |
| **Функции** | `CALL`, `RETURN`, `PARAM` |
| **Данные** | `MOVE` |

### Типы операндов

- **Временные переменные**: `t1`, `t2`, ...
- **Переменные**: `x`, `y`, ...
- **Литералы**: `42`, `3.14`, `true`, `"hello"`
- **Метки**: `L1`, `L2`, ...
- **Адреса**: `[t1]`, `[t2+4]`

### Команды IR

```bash
# Генерация IR в текстовом формате
cargo run -- ir --input factorial.src --ir-format text

# Генерация IR с оптимизациями
cargo run -- ir --input factorial.src --ir-format text --optimize

# Статистика IR
cargo run -- ir --input factorial.src --stats

# Визуализация графа потока управления (CFG)
cargo run -- ir --input factorial.src --ir-format dot --output cfg.dot
dot -Tpng cfg.dot -o cfg.png

# Вывод в JSON
cargo run -- ir --input factorial.src --ir-format json --output ir.json
```

### Примеры IR

**If-else:**
```
function main: int ()
  locals:
    x: int
    y: int

  L1:
    t1 = CMP_GT x, 0
    JUMP_IF_NOT t1, L2
    JUMP L3

  L2:
    y = MOVE 10
    JUMP L4

  L3:
    y = MOVE 20
    JUMP L4

  L4:
    RETURN y
```

**While цикл:**
```
function main: int ()
  locals:
    i: int
    sum: int

  L2:
    t1 = CMP_LT i, 5
    JUMP_IF_NOT t1, L4
    JUMP L3

  L3:
    t2 = ADD sum, i
    sum = MOVE t2
    t3 = ADD i, 1
    i = MOVE t3
    JUMP L2

  L4:
    RETURN sum
```

**Рекурсивный факториал:**
```
function factorial: int (int n)
  L1:
    t1 = CMP_LE n, 1
    JUMP_IF t1, L2
    JUMP L3

  L2:
    RETURN 1

  L3:
    t2 = SUB n, 1
    PARAM 0, t2
    t3 = CALL factorial, t2
    t4 = MUL n, t3
    RETURN t4
```

**Switch:**
```
function main: int ()
  locals:
    x: int
    result: int

  L1:
    x = MOVE 2
    result = MOVE 0
    t1 = CMP_EQ x, 1
    JUMP_IF t1, L_case1
    t2 = CMP_EQ x, 2
    JUMP_IF t2, L_case2
    JUMP L_default

  L_case1:
    result = MOVE 10
    JUMP L_switch_end

  L_case2:
    result = MOVE 20
    JUMP L_switch_end

  L_default:
    result = MOVE 0
    JUMP L_switch_end

  L_switch_end:
    RETURN result
```

## Управляющие конструкции

### If-Else

Компилятор генерирует правильные последовательности условных переходов с уникальными метками для вложенных условий. Операторы отношения (`<`, `<=`, `>`, `>=`, `==`, `!=`) транслируются в соответствующие инструкции `set*` и условные переходы.

### Switch-Case-Default

Оператор выбора реализован через цепочку сравнений `CMP_EQ` с переходами `JUMP_IF` к блокам case. Если ни один case не совпал, выполняется блок default (или сразу переход к концу switch).

### Циклы While и For

Циклы `while` и `for` транслируются в структуры с метками:
- **While**: `.while_cond` → условие → `.while_body` → `jmp .while_cond` → `.while_end`
- **For**: транслируется в эквивалент `init; while (cond) { body; update; }`

For-циклы с константными границами и инкрементом на 1 автоматически оптимизируются в счетные циклы.

### Break и Continue

- **Break**: генерирует `JUMP` к `.while_end` или `.for_end` текущего цикла
- **Continue**: генерирует `JUMP` к `.while_cond` (для while) или `.for_update` (для for)

Семантический анализатор проверяет, что `break` и `continue` используются только внутри циклов (`loop_depth > 0`).

## Короткая схема вычислений

Логические операторы `&&` и `||` реализуют короткую схему (short-circuit evaluation):

- **AND (`&&`)**: `result = 0; if (!left) goto merge; result = right; merge:`
- **OR (`||`)**: `result = 1; if (left) goto merge; result = right; merge:`

Это позволяет безопасно писать код с потенциально опасными операциями:
```c
if (a != 0 && b / a > 2) { ... }  // деление только если a != 0
if (x == 0 || 100 / x > 5) { ... } // деление только если x != 0
```

## Семантический анализ

Семантический анализатор проверяет корректность программы на уровне типов и областей видимости.

### Новые проверки

| Проверка           | Описание                                                           |
|--------------------|--------------------------------------------------------------------|
| **Break/Continue** | Проверка, что `break`/`continue` используются только внутри циклов |
| **Switch**         | Проверка типа выражения switch и уникальности case-значений        |
| **Массивы**        | Проверка объявления массивов и доступа по индексу                  |

### Команды семантического анализа

```bash
# Базовый семантический анализ
cargo run -- semantic --input program.src

# Вывод таблицы символов
cargo run -- semantic --input program.src --show-symbols

# Вывод с размерами и смещениями
cargo run -- semantic --input program.src --show-symbols --show-layout

# Вывод декорированного AST (с типами)
cargo run -- semantic --input program.src --show-ast
```

### Декорированное AST

```bash
cargo run -- semantic --input factorial.src --show-ast
```

Вывод:
```
Program [global scope]:
  Symbol Table:
    factorial: fn(int) -> int функция
    main: fn() -> int функция

  FunctionDecl: factorial -> int [line 1]:
    Parameters:
      - n: int
    Body [type checked]:
      Block [line 1]:
        IfStmt [line 2]:
          Condition: (n <= 1) [type: bool]
          Then branch:
            Block [line 2]:
              Return: 1 [type: int]
          Else branch:
            Block [line 4]:
              Return: (n * factorial((n - 1))) [type: int]
```

## Вывод типов (var)

Ключевое слово `var` позволяет компилятору автоматически определить тип переменной из инициализатора.

### Синтаксис
```c
var x = 42;      // выводится int
var y = 3.14;    // выводится float
var z = true;    // выводится bool
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

## Оптимизации IR

### Поддерживаемые оптимизации

| Оптимизация                  | Пример                                  |
|------------------------------|-----------------------------------------|
| **Свертка констант**         | `3 + 4 → 7`                             |
| **Алгебраические упрощения** | `x + 0 → x`, `x * 1 → x`, `x * 0 → 0`   |
| **Удаление мертвого кода**   | Удаление неиспользуемых переменных      |
| **Сцепление переходов**      | `JUMP L1; L1: JUMP L2 → JUMP L2`        |
| **ЛИЦМ (LICM)**              | Вынос инвариантных вычислений из циклов |

### Пример оптимизации

**До оптимизации:**
```
t1 = ADD x, 0
t2 = MUL t1, 1
t3 = CMP_GT t2, 5
JUMP_IF t3, L1
JUMP L2
```

**После оптимизации:**
```
t1 = CMP_GT x, 5
JUMP_IF t1, L1
JUMP L2
```

### Запуск оптимизаций

```bash
# Генерация IR с оптимизациями
cargo run -- ir --input program.src --optimize

# Подробный отчет об оптимизациях
cargo run -- ir --input program.src --optimize --verbose
```

## Демонстрации

### Демонстрация Sprint 6
```bash
make sprint6-demo
# или
cargo run --example sprint6_demo
```

### Демонстрация кодогенерации
```bash
make codegen-demo
# или
cargo run -- codegen --input examples/factorial.src --output factorial.asm --stats
```

### Демонстрация IR генерации
```bash
make ir-demo
# или
cargo run -- ir --input examples/factorial.src --ir-format text --stats
```

### Демонстрация оптимизаций
```bash
make optimization-demo
# или
cargo run -- ir --input examples/simple_arith.src --optimize --verbose
```

### Демонстрация инкрементов
```bash
make inc-demo
cargo run -- inc-demo
```

### Демонстрация семантического анализа
```bash
make semantic-demo
cargo run -- semantic --input examples/errors.src --show-symbols
```

### Демонстрация вывода типов (var)
```bash
make var-demo
cargo run -- semantic --input examples/var_demo.src --show-symbols --show-layout
```

### Демонстрация восстановления после ошибок
```bash
make error-demo
cargo run -- error-demo --input examples/errors.src --max-errors 50
```

### LL(1) анализ грамматики
```bash
make ll1-demo
cargo run -- ll1 --show-first --show-follow
```

### Полный пайплайн
```bash
make full-pipeline
cargo run -- full --input examples/full_demo.src --show-metrics
```

## Визуализация AST и CFG

### Текстовый формат AST
```bash
cargo run -- parse --input examples/factorial.src
```

### Декорированное AST с типами
```bash
cargo run -- semantic --input examples/factorial.src --show-ast
```

### Graphviz DOT формат для AST
```bash
cargo run -- parse --input examples/struct.src --ast-format dot --output ast.dot
dot -Tpng ast.dot -o ast.png
```

### Graphviz DOT формат для CFG
```bash
cargo run -- ir --input examples/if_else.src --ir-format dot --output cfg.dot
dot -Tpng cfg.dot -o cfg.png
```

### JSON формат
```bash
# AST в JSON
cargo run -- parse --input examples/hello.src --ast-format json --output ast.json

# IR в JSON
cargo run -- ir --input examples/factorial.src --ir-format json --output ir.json
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
make test-ir
make test-ir-opt
make test-codegen
make test-control-flow
make test-abi              
make test-all              
```

### Статистика тестов

| Категория             | Количество |
|-----------------------|:----------:|
| Unit tests (lib)      |     52     |
| Parser tests          |     27     |
| Semantic tests        |     24     |
| Codegen tests         |     24     |
| Control flow tests    |     24     |
| Integration tests     |     33     |
| IR tests              |     7      |
| IR optimization tests |     3      |
| IR golden tests       |     8      |
| ABI compliance tests  |     5      |
| Lexer tests           |     7      |
| LL(1) tests           |     2      |
| Preprocessor tests    |     8      |
| Bugs tests            |     4      |
| Doc-tests             |     28     |
| **Всего**             |  **260+**  |

### Golden Tests

Golden tests автоматически сравнивают сгенерированный IR с эталонными файлами:

```bash
# Первый запуск - создает эталонные файлы
cargo test --test ir_golden_tests

# Последующие запуски - проверяют соответствие
cargo test --test ir_golden_tests
```

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
   │    Лексер    │  ← Токенизация
   └──────────────┘
          │
          ▼
   ┌──────────────┐
   │    Парсер    │  ← Рекурсивный спуск (LL(1))
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
          ▼
   ┌──────────────┐
   │ IR Generator │  ← Генерация промежуточного представления
   └──────────────┘
          │
          ├──────────▶ Текстовый IR
          │
          ├──────────▶ Graphviz DOT (CFG)
          │
          ├──────────▶ JSON
          │
          ▼
   ┌──────────────┐
   │ Оптимизатор  │  ← Свертка констант, удаление мертвого кода, LICM
   └──────────────┘
          │
          ▼
   Оптимизированный IR
          │
          ▼
   ┌──────────────┐
   │ x86-64 Code  │  ← Генерация ассемблера
   │  Generator   │
   └──────────────┘
          │
          ├──────────▶ x86-64 ассемблер
          │
          ├──────────▶ Стековый фрейм
          │
          ├──────────▶ ABI compliance
          │
          ▼
        EXE файл
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
- Поддержка `var`, `switch`, `break`, `continue`, массивов

#### Семантический анализатор (`src/semantic/`)
- Иерархическая таблица символов
- Полная система типов
- Проверка объявлений и областей видимости
- Проверка типов выражений
- Проверка вызовов функций
- Проверка структур и доступа к полям
- Вывод типов для `var`
- Проверка `break`/`continue` внутри циклов
- Вычисление размеров и смещений в памяти

#### Генератор IR (`src/ir/`)
- Преобразование AST в трехадресный код
- Построение базовых блоков и CFG
- Поддержка всех конструкций языка
- Сохранение информации о типах
- Генерация короткой схемы для `&&` и `||`

#### Оптимизатор IR (`src/ir/peephole_optimizer.rs`)
- Свертка констант
- Алгебраические упрощения
- Удаление мертвого кода
- Сцепление переходов
- Вынос инвариантных вычислений из циклов (LICM)

#### Генератор x86-64 кода (`src/codegen/`)
- `x86_generator.rs` - трансляция IR в ассемблер
- `control_flow_generator.rs` - генератор управляющих конструкций
- `expression_generator.rs` - генератор выражений
- `label_manager.rs` - менеджер меток
- `stack_frame.rs` - управление стековым фреймом
- `register_allocator.rs` - распределение регистров
- `abi.rs` - System V AMD64 ABI константы

#### Рантайм-библиотека (`src/runtime/`)
- `runtime.asm` - ассемблерные функции (print_int, exit, etc.)

## LL(1) анализ

### First и Follow множества

```bash
cargo run -- ll1 --show-first --show-follow
```

Вывод:
```
First множества:
  First(E) = { "id" }
  First(E') = { "+" }
  First(T) = { "id" }
  First(T') = { "*" }
  First(F) = { "id" }

Follow множества:
  Follow(E) = { ")" }
  Follow(E') = { ")" }
  Follow(T) = { "+" }
  Follow(T') = { "+" }
  Follow(F) = { "*" }

Грамматика является LL(1)
```

## Восстановление после ошибок

### Стратегии восстановления

1. **Панический режим** - пропуск до точки синхронизации
2. **Уровень фраз** - вставка/удаление токенов
3. **Продукции для ошибок** - специальные правила

### Метрики ошибок

```bash
cargo run -- error-demo --input examples/errors.src --max-errors 10
```

Вывод:
```
Метрики ошибок:
  Обнаружено ошибок: 10
  Фактических ошибок: 2
  Предотвращено каскадных: 8
  Успешно восстановлено: 10
  Качество восстановления: 100.0%
```

## Поддерживаемые конструкции языка

| Категория        | Конструкции                                                  |
|------------------|--------------------------------------------------------------|
| **Функции**      | Объявление, параметры, возвращаемые типы (`->`), рекурсия    |
| **Структуры**    | Определение, поля, доступ (`.`), вложенность                 |
| **Массивы**      | Статические массивы, доступ по индексу `[]`                  |
| **Переменные**   | Объявление, инициализация, присваивание, вывод типов (`var`) |
| **Управление**   | `if-else`, `switch-case-default`, `while`, `for`             |
| **Переходы**     | `break`, `continue`                                          |
| **Инкременты**   | `++x`, `x++`, `--x`, `x--`                                   |
| **Выражения**    | Арифметика, сравнения, логика с короткой схемой              |
| **Препроцессор** | `#define`, `#ifdef`, `#ifndef`, `#else`, `#endif`            |
| **Типы**         | `int`, `float`, `bool`, `void`, `string`, `struct`, `var`    |

## Команда

- **Владимир (Feronski)** - Ведущий разработчик

## Полезные ссылки

- [Спецификация языка MiniC](docs/language_spec.md)
- [Формальная грамматика](docs/grammar.md)
- [Примеры использования](examples/)
- [Чек-лист спринтов](docs/CHECKLIST.md)

**Версия:** 0.6.0
**Дата релиза:** Май 2026