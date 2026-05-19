# Mini Compiler

Минимальный компилятор C-подобного языка, написанный на Rust с полной поддержкой LL(1) грамматики, семантическим анализом, генерацией промежуточного представления (IR) и генерацией x86-64 ассемблерного кода.

## Оглавление

- [Особенности](#особенности)
- [Новые возможности Sprint 7](#новые-возможности-sprint-7)
- [Структура проекта](#структура-проекта)
- [Установка и сборка](#установка-и-сборка)
- [Makefile команды](#makefile-команды)
- [Быстрый старт](#быстрый-старт)
- [Использование CLI](#использование-cli)
- [Генерация промежуточного представления (IR)](#генерация-промежуточного-представления-ir)
- [Генерация x86-64 ассемблерного кода](#генерация-x86-64-ассемблерного-кода)
- [Управляющие конструкции](#управляющие-конструкции-sprint-6)
- [Короткая схема вычислений](#короткая-схема-вычислений)
- [Внешние функции и системная интеграция](#внешние-функции-и-системная-интеграция-sprint-7)
- [Массивы и указатели](#массивы-и-указатели-sprint-7)
- [Семантический анализ](#семантический-анализ)
- [Вывод типов (var)](#вывод-типов-var)
- [Оптимизации IR](#оптимизации-ir)
- [Function Inlining](#function-inlining-sprint-7-stretch-goal)
- [Демонстрации](#демонстрации)
- [Визуализация AST и CFG](#визуализация-ast-и-cfg)
- [Тестирование](#тестирование)
- [Документация](#документация)
- [Архитектура](#архитектура)
- [LL(1) анализ](#ll1-анализ)
- [Восстановление после ошибок](#восстановление-после-ошибок)
- [Поддерживаемые конструкции языка](#поддерживаемые-конструкции-языка)
- [Команда](#команда)
- [Полезные ссылки](#полезные-ссылки)

## Особенности

### Технические характеристики
- **Язык**: Rust 2024 edition
- **Целевая архитектура**: x86-64
- **ABI**: System V AMD64
- **Ассемблер**: NASM
- **Линковка**: GCC-совместимая (libc, статическая/динамическая)
- **Обработка ошибок**: Детальные сообщения с восстановлением
- **Поддержка кодировок**: UTF-8
- **Окончания строк**: Unix (`\n`) и Windows (`\r\n`)

### Новые возможности Sprint 7

| Возможность                    | Описание                                                         |
|--------------------------------|------------------------------------------------------------------|
| **Внешние функции**            | `extern` декларации, вызов `printf`, `scanf`, `malloc` из libc   |
| **Variadic аргументы**         | Поддержка `...` для функций вроде `printf`                       |
| **Указатели**                  | Типы `char*`, `int*`, разыменование `*ptr`, взятие адреса `&var` |
| **Массивы с инициализацией**   | `int arr[5] = {1, 2, 3, 4, 5}`                                  |
| **Массивы как параметры**      | `void foo(int arr[], int size)`                                  |
| **System V AMD64 ABI**         | Полное соответствие для внешних вызовов (8+ аргументов)          |
| **Инлайнинг функций**          | Встраивание маленьких функций (Stretch Goal)                     |
| **Глобальный DCE**             | Анализ использования переменных по всем блокам                   |

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
│   │   ├── token.rs                  # Токены (20+ ключевых слов)
│   │   └── position.rs               # Позиция в исходном коде
│   ├── lexer/                        # Лексический анализатор
│   │   ├── mod.rs                    # Основной модуль
│   │   ├── scanner.rs                # Сканер (основная логика)
│   │   └── error.rs                  # Ошибки лексического анализа
│   ├── parser/                       # Парсер
│   │   ├── mod.rs                    # Экспорт модуля
│   │   ├── parser.rs                 # Рекурсивный спуск
│   │   ├── ast.rs                    # Структуры AST
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
│   │   ├── peephole_optimizer.rs     # Оптимизации IR
│   │   └── inline_optimizer.rs       # Инлайнинг функций (Sprint 7)
│   ├── codegen/                      # Генерация x86-64 кода
│   │   ├── mod.rs                    # Экспорт модуля
│   │   ├── x86_generator.rs          # Генератор x86-64 ассемблера
│   │   ├── control_flow_generator.rs # Генератор управляющих конструкций
│   │   ├── expression_generator.rs   # Генератор выражений
│   │   ├── label_manager.rs          # Менеджер меток
│   │   ├── stack_frame.rs            # Управление стековым фреймом
│   │   ├── register_allocator.rs     # Аллокатор регистров (linear scan)
│   │   └── abi.rs                    # System V AMD64 ABI константы
│   ├── preprocessor/                 # Препроцессор
│   │   ├── mod.rs                    # Основной модуль
│   │   ├── macros.rs                 # Таблица макросов
│   │   └── error.rs                  # Ошибки препроцессора
│   ├── utils/                        # Вспомогательные функции
│   ├── lib.rs                        # Точка входа библиотеки
│   └── main.rs                       # Точка входа CLI
├── tests/                            # Тестовые файлы
│   ├── control_flow_tests.rs         # Тесты потока управления
│   ├── integration_codegen.rs        # Интеграционные тесты кодогенерации
│   ├── semantic_tests.rs             # Тесты семантического анализа
│   ├── ir_optimization_tests.rs      # Тесты оптимизаций
│   ├── ir_golden_tests.rs            # Golden тесты IR
│   ├── abi_compliance_tests.rs       # ABI compliance тесты
│   └── ...
├── examples/                         # Демонстрационные файлы
│   └── sprint7_full_demo.src         # Полное демо Sprint 7
├── docs/                             # Документация
│   ├── CHECKLIST.md                  # Чек-лист по спринтам
│   ├── language_spec.md              # Спецификация языка
│   └── grammar.md                    # Формальная грамматика
├── Cargo.toml                        # Конфигурация Cargo
├── Makefile                          # Система сборки
└── README.md                         # Этот файл
```

## Установка и сборка

### Предварительные требования

- **Rust 1.70 или новее**
- **Cargo** (менеджер пакетов Rust)
- **Git** (для клонирования репозитория)
- **NASM** (для ассемблирования сгенерированного кода)
- **GCC** (для линковки с libc)
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

### Демонстрации Sprint 7 (НОВЫЕ!)
```bash
make sprint7-demo            # Полная демонстрация всех новых возможностей
make inline-demo             # Демонстрация Function Inlining
```

### Демонстрации Sprint 6
```bash
make sprint6-demo            # Полная демонстрация Sprint 6
make switch-demo             # Демонстрация Switch-case-default
make break-continue-demo     # Демонстрация Break и Continue
make short-circuit-demo      # Демонстрация короткой схемы вычислений
make float-demo              # Демонстрация Float и приведения типов
make array-demo              # Демонстрация массивов
```

### Другие демонстрации
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

### 1. Запуск демонстрации Sprint 7

```bash
make build
make sprint7-demo
```

### 2. Полный пайплайн компиляции

```bash
# Создайте тестовый файл с extern и printf
cat > test.src << 'EOF'
extern int printf(char* format, ...);

int factorial(int n) {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

int main() {
    printf("factorial(5) = %d\n", factorial(5));
    return 0;
}
EOF

# Компиляция с оптимизациями и инлайнингом
cargo run -- codegen --input test.src --output test.asm --inline --optimize --stats

# Сборка и запуск
nasm -f elf64 test.asm -o test.o
gcc -no-pie -o test test.o
./test
```

### 3. Быстрый запуск демонстраций Sprint 6

```bash
make build
make sprint6-demo
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
cargo run -- ir --input file.src --inline --ir-format text          # с инлайнингом

# Генерация x86-64 ассемблерного кода
cargo run -- codegen --input file.src --output output.asm
cargo run -- codegen --input file.src --output output.asm --optimize
cargo run -- codegen --input file.src --output output.asm --inline   # с инлайнингом
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
| **Указатели**          | `AddrOf → lea`, `Gep`                                          |
| **Преобразование**     | `IntToFloat → cvtsi2sd`, `FloatToInt → cvttsd2si`              |
| **Управление потоком** | `JUMP → jmp`, `JUMP_IF → jnz`, `CALL → call`, `RETURN → ret`   |

### System V AMD64 ABI

**Соглашение о вызовах:**
- Первые 6 целочисленных аргументов: `RDI`, `RSI`, `RDX`, `RCX`, `R8`, `R9`
- Аргументы 7+ передаются через стек
- Возвращаемое значение: `RAX` (целые), `XMM0` (float)
- Для variadic функций: `AL` = количество используемых SSE регистров (0 для целых)

**Использование регистров:**
- **Caller-saved**: `RAX`, `RCX`, `RDX`, `RSI`, `RDI`, `R8-R11`
- **Callee-saved**: `RBX`, `RSP`, `RBP`, `R12-R15`

**Выравнивание стека:** 16 байт перед каждым `call`

### Примеры генерации

**Простая функция:**
```c
fn main() -> int {
    return 42;
}
```

```asm
section .text
global main

main:
    push rbp
    mov rbp, rsp
    mov rax, 42
    mov rsp, rbp
    pop rbp
    ret
```

**Внешняя функция с variadic аргументами:**
```c
extern int printf(char* format, ...);

int main() {
    int x = 42;
    printf("The answer is %d\n", x);
    return 0;
}
```

```asm
section .data
L_str0: db "The answer is %d", 10, 0

section .text
extern printf
global main

main:
    push rbp
    mov rbp, rsp
    sub rsp, 16
    mov dword [rbp-4], 42
    lea rdi, [rel L_str0]
    mov esi, [rbp-4]
    xor eax, eax
    call printf
    xor eax, eax
    mov rsp, rbp
    pop rbp
    ret
```

**If-Else:**
```c
if (x > 0) {
    y = 10;
} else {
    y = 20;
}
```

```asm
    mov eax, [rbp-8]
    cmp eax, 0
    jle .Lelse
    mov dword [rbp-12], 10
    jmp .Lendif
.Lelse:
    mov dword [rbp-12], 20
.Lendif:
```

**While цикл:**
```c
while (i < 10) {
    sum = sum + i;
    i = i + 1;
}
```

```asm
.Lwhile_cond:
    mov eax, [rbp-4]
    cmp eax, 10
    jge .Lwhile_end
    mov eax, [rbp-8]
    add eax, [rbp-4]
    mov [rbp-8], eax
    add dword [rbp-4], 1
    jmp .Lwhile_cond
.Lwhile_end:
```

**Короткая схема (Short-Circuit):**
```c
if (a != 0 && b / a > 2) {
    result = 1;
}
```

```asm
    mov eax, [rbp-8]
    cmp eax, 0
    je .Lfalse
    mov eax, [rbp-12]
    cdq
    idiv dword [rbp-8]
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
switch (x) {
    case 1: result = 10;
    case 2: result = 20;
    default: result = 0;
}
```

```asm
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

# С оптимизациями и инлайнингом
cargo run -- codegen --input factorial.src --output factorial.asm --inline --optimize

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
sub rsp, N      ; Выделение места для локальных переменных и ALLOCA (выровнено по 16)
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
| **Указатели** | `AddrOf` |
| **Преобразование** | `IntToFloat`, `FloatToInt` |
| **Управление потоком** | `JUMP`, `JUMP_IF`, `JUMP_IF_NOT`, `LABEL`, `PHI`, `CMP_JMP` |
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

# Генерация IR с оптимизациями и инлайнингом
cargo run -- ir --input factorial.src --ir-format text --optimize --inline

# Статистика IR
cargo run -- ir --input factorial.src --stats

# Визуализация графа потока управления (CFG)
cargo run -- ir --input factorial.src --ir-format dot --output cfg.dot
dot -Tpng cfg.dot -o cfg.png
```

### Примеры IR

**If-else:**
```
function main: int ()
  locals: x: int, y: int
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
  locals: i: int, sum: int
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

**Инлайнинг square(5):**
```
// До инлайнинга:
    PARAM 0, 5
    t1 = CALL square, 5
    a = MOVE t1

// После инлайнинга:
    x_1 = MOVE 5
    JUMP square_body_inl1
  __after_inline_0:
    a = MOVE t2
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

### Break и Continue

- **Break**: генерирует `JUMP` к `.while_end` или `.for_end` текущего цикла
- **Continue**: генерирует `JUMP` к `.while_cond` (для while) или `.for_update` (для for)

## Короткая схема вычислений

Логические операторы `&&` и `||` реализуют короткую схему (short-circuit evaluation):

- **AND (`&&`)**: `result = 0; if (!left) goto merge; result = right; merge:`
- **OR (`||`)**: `result = 1; if (left) goto merge; result = right; merge:`

```c
if (a != 0 && b / a > 2) { ... }  // деление только если a != 0
```

## Внешние функции и системная интеграция (Sprint 7)

### Поддержка extern

Компилятор поддерживает объявление внешних функций через ключевое слово `extern`:

```c
extern int printf(char* format, ...);
extern int scanf(char* format, ...);
extern void* malloc(int size);
extern void free(void* ptr);
```

Для внешних функций генерируется `extern` декларация в ассемблере, и линковка с libc происходит автоматически через GCC.

### Variadic функции

Поддерживаются функции с переменным числом аргументов (`...`):

```c
extern int printf(char* format, ...);

int main() {
    printf("Sum: %d + %d = %d\n", 10, 20, 30);
    printf("Multiple: %d %d %d %d %d %d\n", 1, 2, 3, 4, 5, 6);
    return 0;
}
```

Особенности реализации:
- Корректная установка `eax=0` для variadic вызовов
- Соблюдение System V AMD64 ABI (первые 6 аргументов в регистрах, остальные в стеке)
- Поддержка до 8+ аргументов (протестировано на `sum8(a,b,c,d,e,f,g,h)`)

### Линковка с C library

Сгенерированный ассемблер совместим с GCC линковкой:

```bash
nasm -f elf64 program.asm -o program.o
gcc -no-pie program.o -o program
./program
```

## Массивы и указатели (Sprint 7)

### Стековые массивы с инициализацией

```c
int main() {
    int arr[5] = {10, 20, 30, 40, 50};  // инициализация
    int x = arr[2];                       // доступ: x = 30
    arr[0] = 99;                          // модификация
    return arr[0];
}
```

Массивы выделяются на стеке через `ALLOCA` с правильным выравниванием. Инициализация происходит поэлементно через `ArrayStore`.

### Массивы как параметры функций

```c
void bubble_sort(int arr[], int size) {
    for (int i = 0; i < size - 1; i = i + 1) {
        for (int j = 0; j < size - i - 1; j = j + 1) {
            if (arr[j] > arr[j + 1]) {
                int tmp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = tmp;
            }
        }
    }
}
```

Массивы передаются как указатели (через `int*`). Доступ к элементам через `ArrayLoad`/`ArrayStore`.

### Указатели и адресная арифметика

```c
void swap(int* a, int* b) {
    int temp = *a;    // разыменование (IR: LOAD)
    *a = *b;          // (IR: STORE)
    *b = temp;
}

int main() {
    int x = 10, y = 20;
    swap(&x, &y);     // взятие адреса (IR: AddrOf)
    // x=20, y=10
    return 0;
}
```

### Доступ к элементам массива по указателю

```c
int main() {
    int arr[5] = {1, 2, 3, 4, 5};
    int* p = &arr[0];
    int second = *(p + 1);  // доступ через арифметику
    return second;
}
```

`&arr[i]` вычисляется как `arr + i * 8` (IR: `MUL i, 8; ADD arr, offset`).

## Семантический анализ

Семантический анализатор проверяет корректность программы на уровне типов и областей видимости.

### Проверки Sprint 7

| Проверка           | Описание                                                           |
|--------------------|--------------------------------------------------------------------|
| **Указатели**      | Проверка совместимости `int*` и `char*`, `*ptr` типов              |
| **Variadic**       | Проверка минимального количества аргументов для `...` функций      |
| **Типы строк**     | Строковые литералы `"hello"` имеют тип `char*` (указатель)         |
| **Присваивание**   | `float = int` разрешено (расширение), `int = float` запрещено      |

### Команды семантического анализа

```bash
# Базовый семантический анализ
cargo run -- semantic --input program.src

# Вывод таблицы символов
cargo run -- semantic --input program.src --show-symbols

# Вывод с размерами и смещениями
cargo run -- semantic --input program.src --show-symbols --show-layout
```

## Вывод типов (var)

Ключевое слово `var` позволяет компилятору автоматически определить тип переменной из инициализатора.

```c
var x = 42;      // int
var y = 3.14;    // float
var z = true;    // bool
var s = "hello"; // char* (указатель на строку)
```

## Оптимизации IR

### Поддерживаемые оптимизации

| Оптимизация                  | Пример                                  |
|------------------------------|-----------------------------------------|
| **Свертка констант**         | `3 + 4 → 7`                             |
| **Алгебраические упрощения** | `x + 0 → x`, `x * 1 → x`, `x * 0 → 0`   |
| **Удаление мертвого кода**   | Удаление неиспользуемых переменных      |
| **Глобальный DCE**           | Анализ использования по всем блокам     |
| **Сохранение вызовов**       | `Call` инструкции не удаляются DCE      |
| **ЛИЦМ (LICM)**              | Вынос инвариантных вычислений из циклов |

### Команды

```bash
# IR с оптимизациями
cargo run -- ir --input program.src --optimize

# Кодогенерация с оптимизациями и инлайнингом
cargo run -- codegen --input program.src --output out.asm --optimize --inline --stats
```

## Function Inlining (Sprint 7, Stretch Goal)

Инлайнинг встраивает тело маленьких функций непосредственно в место вызова, устраняя накладные расходы на `call`/`ret`.

### Критерии для инлайнинга

| Критерий              | Порог                        |
|-----------------------|------------------------------|
| Размер функции        | ≤ 20 IR-инструкций           |
| Циклы                 | Не содержит `for`/`while`    |
| Указатели             | Не содержит `AddrOf`         |
| Рекурсия              | Не рекурсивная               |
| Внешние               | Не `extern`                  |
| `swap`                | Исключена (содержит `Store`) |

### Пример

**Исходный код:**
```c
int square(int x) { return x * x; }
int cube(int x) { return x * x * x; }

int main() {
    int a = square(5);   // будет встроено
    int b = cube(3);     // будет встроено
    return a + b;
}
```

**IR до инлайнинга:**
```
    PARAM 0, 5
    t1 = CALL square, 5
    a = MOVE t1
    PARAM 0, 3
    t2 = CALL cube, 3
    b = MOVE t2
```

**IR после инлайнинга:**
```
    x_1 = MOVE 5
    JUMP square_body_inl1
  __after_inline_0:
    a = MOVE t1
    x_2 = MOVE 3
    JUMP cube_body_inl2
  __after_inline_1:
    b = MOVE t2

  square_body_inl1:
    t1 = MUL 5, 5
    t2 = MOVE t1
    JUMP __after_inline_0

  cube_body_inl2:
    t1 = MUL 3, 3
    t2 = MUL t1, 3
    JUMP __after_inline_1
```

### Запуск

```bash
# Просмотр IR с инлайнингом
cargo run -- ir --input program.src --inline --ir-format text

# Компиляция с инлайнингом и оптимизациями
cargo run -- codegen --input program.src --output out.asm --inline --optimize --stats
```

### Демонстрация инлайнинга

```bash
make inline-demo
```

## Демонстрации

### Демонстрация Sprint 7
```bash
make sprint7-demo
```

Вывод демонстрирует: extern printf, variadic, стeковые массивы, указатели, swap, сортировку пузырьком, рекурсивную сумму, инлайнинг, оптимизации.

### Демонстрация Sprint 6
```bash
make sprint6-demo
```

### Демонстрация инлайнинга
```bash
make inline-demo
```

### Другие демонстрации
```bash
make ast-demo                # Визуализация AST
make ir-demo                 # Демонстрация IR
make codegen-demo            # Демонстрация кодогенерации
make optimization-demo       # Демонстрация оптимизаций
make semantic-demo           # Семантический анализ
make var-demo                # Вывод типов (var)
make inc-demo                # Инкременты
make error-demo              # Восстановление после ошибок
make ll1-demo                # LL(1) анализ
make full-pipeline           # Полный пайплайн
```

## Визуализация AST и CFG

```bash
# AST в DOT
cargo run -- parse --input examples/struct.src --ast-format dot --output ast.dot
dot -Tpng ast.dot -o ast.png

# CFG в DOT
cargo run -- ir --input examples/if_else.src --ir-format dot --output cfg.dot
dot -Tpng cfg.dot -o cfg.png

# AST в JSON
cargo run -- parse --input examples/hello.src --ast-format json --output ast.json
```

## Тестирование

### Запуск тестов

```bash
# Все тесты
cargo test

# Через Makefile
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
| Unit tests (lib)      |     48     |
| Parser tests          |     26     |
| Semantic tests        |     24     |
| Codegen tests         |     24     |
| Control flow tests    |     24     |
| Integration tests     |     33     |
| IR tests              |      7     |
| IR optimization tests |      3     |
| IR golden tests       |      8     |
| ABI compliance tests  |      5     |
| LL(1) tests           |      2     |
| Bugs tests            |      4     |
| Preprocessor tests    |      8     |
| Lexer tests           |      7     |
| **Всего**             |  **~220** |

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
          ▼
   ┌──────────────┐
   │ IR Generator │  ← Генерация промежуточного представления
   └──────────────┘
          │
          ├── Инлайнинг (Inline Optimizer)
          ├── Peephole оптимизации
          │
          ▼
   ┌──────────────┐
   │ x86-64 Code  │  ← Генерация ассемблера
   │  Generator   │     (с linear scan регистровым аллокатором)
   └──────────────┘
          │
          ▼
        EXE файл
```

## LL(1) анализ

```bash
cargo run -- ll1 --show-first --show-follow
```

## Восстановление после ошибок

- Панический режим
- Уровень фраз (вставка/удаление токенов)
- Продукции для ошибок

## Поддерживаемые конструкции языка

| Категория        | Конструкции                                                            |
|------------------|------------------------------------------------------------------------|
| **Функции**      | `fn`, C-стиль, параметры, возвращаемые типы, рекурсия                  |
| **Внешние**      | `extern`, variadic (`...`), вызов libc                                 |
| **Массивы**      | Стековые `int arr[N]`, инициализация `{a,b,c}`, параметры `int arr[]`  |
| **Указатели**    | `int*`, `char*`, `*ptr`, `&var`, `&arr[i]`                             |
| **Структуры**    | Определение, поля, доступ                                             |
| **Управление**   | `if-else`, `switch-case-default`, `while`, `for`                       |
| **Переходы**     | `break`, `continue`                                                    |
| **Инкременты**   | `++x`, `x++`, `--x`, `x--`                                             |
| **Логика**       | `&&`, `||`, `!` с короткой схемой                                      |
| **Препроцессор** | `#define`, `#ifdef`, `#ifndef`, `#else`, `#endif`                      |
| **Типы**         | `int`, `float`, `bool`, `void`, `char`, `string`, `struct`, `var`      |
| **Оптимизации**  | Свёртка констант, DCE, алгебраические упрощения, инлайнинг            |

## Команда

- **Владимир (Feronski)** - Ведущий разработчик

## Полезные ссылки

- [Спецификация языка MiniC](docs/language_spec.md)
- [Формальная грамматика](docs/grammar.md)
- [Примеры использования](examples/)
- [Чек-лист спринтов](docs/CHECKLIST.md)

**Версия:** 0.7.0
**Дата релиза:** Май 2026