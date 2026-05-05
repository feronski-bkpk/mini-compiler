.PHONY: all build release check test test-lexer test-parser test-preprocessor test-semantic \
        test-ir test-ir-opt test-ir-golden test-integration test-ll1 test-all test-codegen \
        test-codegen-unit test-codegen-integration test-abi test-register clean docs help \
        ast-demo ir-demo semantic-demo var-demo inc-demo error-demo ll1-demo optimization-demo \
        codegen-demo full-pipeline sprint6-demo switch-demo break-continue-demo short-circuit-demo \
        float-demo array-demo lint fmt fmt-check udeps bench coverage graphviz-check \
        install-deps create-test-files

UNAME_S := $(shell uname -s 2>/dev/null || echo Windows)
ifeq ($(UNAME_S),Windows_NT)
    TARGET := x86_64-pc-windows-gnu
    CARGO := cargo --target $(TARGET)
    RM := del /Q
    RMRF := if exist $(1) rmdir /S /Q $(1)
    CP := copy
    EXE := .exe
else
    TARGET :=
    CARGO := cargo
    RM := rm -f
    RMRF := rm -rf
    CP := cp
    EXE :=
endif

CARGO_FLAGS =
RUSTFLAGS = -D warnings

all: check test build

build:
	@echo "Сборка проекта..."
	$(CARGO) build $(CARGO_FLAGS)

release:
	@echo "Сборка в режиме релиза..."
	$(CARGO) build --release $(CARGO_FLAGS)

check:
	@echo "Проверка кода..."
	$(CARGO) check $(CARGO_FLAGS)

# === Тестирование ===
test:
	@echo "Запуск всех тестов..."
	$(CARGO) test $(CARGO_FLAGS) -- --nocapture

test-unit:
	@echo "Запуск unit тестов..."
	$(CARGO) test $(CARGO_FLAGS) --lib -- --nocapture

test-lexer:
	@echo "Запуск тестов лексического анализатора..."
	$(CARGO) test $(CARGO_FLAGS) lexer -- --nocapture

test-parser:
	@echo "Запуск тестов парсера..."
	$(CARGO) test $(CARGO_FLAGS) parser -- --nocapture

test-preprocessor:
	@echo "Запуск тестов препроцессора..."
	$(CARGO) test $(CARGO_FLAGS) preprocessor -- --nocapture

test-semantic:
	@echo "Запуск семантических тестов..."
	$(CARGO) test $(CARGO_FLAGS) semantic_tests -- --nocapture

test-ir:
	@echo "Запуск тестов генерации IR..."
	$(CARGO) test $(CARGO_FLAGS) --test ir_tests -- --nocapture

test-ir-opt:
	@echo "Запуск тестов оптимизаций IR..."
	$(CARGO) test $(CARGO_FLAGS) --test ir_optimization_tests -- --nocapture

test-ir-golden:
	@echo "Запуск golden тестов IR..."
	$(CARGO) test $(CARGO_FLAGS) --test ir_golden_tests -- --nocapture

test-codegen:
	@echo "Запуск тестов кодогенерации..."
	$(CARGO) test $(CARGO_FLAGS) --test codegen_tests -- --nocapture

test-codegen-unit:
	@echo "Запуск unit тестов кодогенерации..."
	$(CARGO) test $(CARGO_FLAGS) --test codegen_tests -- --nocapture

test-codegen-integration:
	@echo "Запуск интеграционных тестов кодогенерации (требуется NASM)..."
ifeq ($(UNAME_S),Windows_NT)
	@echo "  Интеграционные тесты на Windows требуют NASM и MinGW"
	$(CARGO) test $(CARGO_FLAGS) --test integration_codegen -- --ignored --nocapture || true
else
	$(CARGO) test $(CARGO_FLAGS) --test integration_codegen -- --ignored --nocapture
endif

test-control-flow:
	@echo "Запуск тестов потока управления (Sprint 6)..."
	$(CARGO) test $(CARGO_FLAGS) --test control_flow_tests -- --nocapture

test-abi:
	@echo "Запуск ABI compliance тестов..."
	$(CARGO) test $(CARGO_FLAGS) --test abi_compliance_tests -- --nocapture

test-register:
	@echo "Запуск тестов аллокатора регистров..."
	$(CARGO) test $(CARGO_FLAGS) --lib codegen::register_allocator -- --nocapture

test-integration:
	@echo "Запуск интеграционных тестов..."
	$(CARGO) test $(CARGO_FLAGS) integration -- --nocapture

test-ll1:
	@echo "Запуск LL(1) тестов..."
	$(CARGO) test $(CARGO_FLAGS) ll1_tests -- --nocapture

test-common:
	@echo "Запуск тестов общих модулей..."
	$(CARGO) test $(CARGO_FLAGS) common -- --nocapture

test-doc:
	@echo "Запуск документационных тестов..."
	$(CARGO) test $(CARGO_FLAGS) --doc

test-all: test-unit test-ir test-ir-opt test-ir-golden test-codegen test-abi test-register test-ll1 test-semantic test-control-flow test-doc
	@echo "Все тесты пройдены!"

# === Качество кода ===
lint:
	@echo "Проверка линтером..."
	$(CARGO) clippy $(CARGO_FLAGS) -- -D warnings

fmt:
	@echo "Форматирование кода..."
	$(CARGO) fmt

fmt-check:
	@echo "Проверка форматирования кода..."
	$(CARGO) fmt -- --check

# === Документация ===
docs:
	@echo "Генерация документации..."
	$(CARGO) doc --no-deps --open

docs-private:
	@echo "Генерация документации (с приватными элементами)..."
	$(CARGO) doc --document-private-items --no-deps --open

# === Очистка ===
clean:
	@echo "Очистка проекта..."
	$(CARGO) clean
	@$(call RMRF,target/ast-examples)
	@$(call RMRF,target/ir-examples)
	@$(call RMRF,target/codegen-examples)
	@$(call RMRF,target/sprint6-examples)
	@echo "Удаление сгенерированных файлов из examples/..."
	@-$(RM) examples\*.asm 2>/dev/null
	@-$(RM) examples\*.o 2>/dev/null
	@-$(RM) examples\*.exe 2>/dev/null
	@-$(RM) examples\*.json 2>/dev/null
	@-$(RM) examples\*.dot 2>/dev/null
	@echo "Удаление мусорных файлов из корня проекта..."
	@-$(RM) *.asm 2>/dev/null
	@-$(RM) *.o 2>/dev/null
	@-$(RM) *.exe 2>/dev/null
	@-$(RM) *.json 2>/dev/null
	@-$(RM) *.dot 2>/dev/null
	@-$(RM) *.src 2>/dev/null
	@-$(RM) test_cf_*.asm 2>/dev/null
	@-$(RM) test_cf_*.o 2>/dev/null
	@-$(RM) test_cf_*.exe 2>/dev/null
	@-$(RM) test_cf_* 2>/dev/null
	@-$(RM) test_output_*.asm 2>/dev/null
	@-$(RM) test_output_*.o 2>/dev/null
	@-$(RM) test_output_*.exe 2>/dev/null
	@-$(RM) test_program_* 2>/dev/null
	@-$(RM) test_program_*.exe 2>/dev/null
	@-$(RM) demo_output.* 2>/dev/null
	@echo "Очистка завершена!"

# === Анализ ===
udeps:
	@echo "Анализ неиспользуемых зависимостей..."
	$(CARGO) udeps

bench:
	@echo "Запуск бенчмарков..."
	$(CARGO) bench

coverage:
	@echo "Измерение покрытия кода..."
	cargo tarpaulin --ignore-tests --out Html

# === Демонстрации ===
ast-demo:
	@echo "Демонстрация визуализации AST..."
	@mkdir -p target/ast-examples
	@echo "fn main() -> int { return 42; }" > target/ast-examples/simple.src
	@echo "struct Point { int x; int y; } fn main() { struct Point p; p.x = 10; return p.x; }" > target/ast-examples/struct.src
	@echo "fn counter() -> int { int x = 5; x++; ++x; x--; --x; return x; }" > target/ast-examples/increment.src
	@echo ""
	@echo "Текстовый формат AST (простая программа):"
	./target/$(TARGET)/debug/minic$(EXE) parse --input target/ast-examples/simple.src
	@echo ""
	@echo "Текстовый формат AST (с инкрементами):"
	./target/$(TARGET)/debug/minic$(EXE) parse --input target/ast-examples/increment.src
	@echo ""
	@echo "Генерация DOT графа..."
	./target/$(TARGET)/debug/minic$(EXE) parse --input target/ast-examples/struct.src --ast-format dot --output target/ast-examples/ast.dot
	@echo "  DOT файл сохранен: target/ast-examples/ast.dot"

ir-demo:
	@echo "Демонстрация генерации промежуточного представления (IR)..."
	@mkdir -p target/ir-examples
	@echo 'fn main() -> int { int x = 5; int y = 10; int z = x + y; return z; }' > target/ir-examples/simple.src
	@echo 'fn main() -> int { int x = 5; if (x > 0) { return 10; } else { return 20; } }' > target/ir-examples/if.src
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } }' > target/ir-examples/factorial.src
	@echo ""
	@echo "=== Текстовый формат IR (простая арифметика) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/simple.src --ir-format text
	@echo ""
	@echo "=== Текстовый формат IR (if-else) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/if.src --ir-format text
	@echo ""
	@echo "=== Текстовый формат IR (рекурсивный факториал) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/factorial.src --ir-format text
	@echo ""
	@echo "=== Статистика IR ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/factorial.src --stats
	@echo ""
	@echo "=== Генерация DOT графа потока управления (CFG) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/if.src --ir-format dot --output target/ir-examples/cfg.dot
	@echo "  DOT файл сохранен: target/ir-examples/cfg.dot"

codegen-demo:
	@echo "Демонстрация генерации x86-64 кода..."
	@mkdir -p target/codegen-examples
	@echo 'fn main() -> int { return 42; }' > target/codegen-examples/simple.src
	@echo 'fn add(int a, int b) -> int { return a + b; } fn main() -> int { return add(5, 3); }' > target/codegen-examples/call.src
	@echo 'fn main() -> int { int sum = 0; int i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } return sum; }' > target/codegen-examples/loop.src
	@echo ""
	@echo "=== Генерация ассемблера (простая функция) ==="
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/codegen-examples/simple.src --output target/codegen-examples/simple.asm
	@cat target/codegen-examples/simple.asm
	@echo ""
	@echo "=== Генерация ассемблера (вызов функции) ==="
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/codegen-examples/call.src --output target/codegen-examples/call.asm
	@echo ""
	@echo "=== Генерация ассемблера (цикл) ==="
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/codegen-examples/loop.src --output target/codegen-examples/loop.asm
	@echo ""
	@echo "=== Статистика кодогенерации ==="
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/codegen-examples/loop.src --output target/codegen-examples/loop.asm --stats

optimization-demo:
	@echo "Демонстрация оптимизаций IR..."
	@mkdir -p target/ir-examples
	@echo 'fn main() -> int { int x = 2 + 3; int y = 5 * 4; int z = 10 - 3; return x + y + z; }' > target/ir-examples/const_fold.src
	@echo 'fn main() -> int { int x = 5; int a = x + 0; int b = x * 1; int c = x * 0; return a + b + c; }' > target/ir-examples/algebraic.src
	@echo 'fn main() -> int { int x = 5; int y = 10; int z = x + y; int w = z * 2; return x; }' > target/ir-examples/dead_code.src
	@echo ""
	@echo "=== Свертка констант (до оптимизации) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/const_fold.src --ir-format text
	@echo ""
	@echo "=== Свертка констант (после оптимизации) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/const_fold.src --ir-format text --optimize --verbose
	@echo ""
	@echo "=== Алгебраические упрощения (до) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/algebraic.src --ir-format text
	@echo ""
	@echo "=== Алгебраические упрощения (после) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/algebraic.src --ir-format text --optimize --verbose
	@echo ""
	@echo "=== Удаление мертвого кода (до) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/dead_code.src --ir-format text
	@echo ""
	@echo "=== Удаление мертвого кода (после) ==="
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/ir-examples/dead_code.src --ir-format text --optimize --verbose

semantic-demo:
	@echo "Демонстрация семантического анализа..."
	@mkdir -p target/ast-examples
	@echo 'fn add(int a, int b) -> int { return a + b; }' > target/ast-examples/correct.src
	@echo 'fn main() -> int { int x = y + 5; return x; }' > target/ast-examples/undeclared.src
	@echo 'fn main() -> int { int x = 3.14; return x; }' > target/ast-examples/type_mismatch.src
	@echo ""
	@echo "=== Корректная программа ==="
	./target/$(TARGET)/debug/minic$(EXE) semantic --input target/ast-examples/correct.src --show-symbols
	@echo ""
	@echo "=== Необъявленная переменная ==="
	./target/$(TARGET)/debug/minic$(EXE) semantic --input target/ast-examples/undeclared.src || true

var-demo:
	@echo "Демонстрация вывода типов (var)..."
	@mkdir -p target/ast-examples
	@echo 'fn main() -> int { var x = 42; var y = 3.14; var z = true; var s = "hello"; return 0; }' > target/ast-examples/var_demo.src
	@echo ""
	./target/$(TARGET)/debug/minic$(EXE) semantic --input target/ast-examples/var_demo.src --show-symbols

inc-demo:
	@echo "Демонстрация инкрементов/декрементов..."
	@mkdir -p target/ast-examples
	@echo 'fn main() -> int { int x = 5; int a = x++; int b = ++x; int c = x--; int d = --x; return x; }' > target/ast-examples/inc.src
	@echo ""
	./target/$(TARGET)/debug/minic$(EXE) parse --input target/ast-examples/inc.src

error-demo:
	@echo "Демонстрация восстановления после ошибок..."
	@mkdir -p target/ast-examples
	@echo 'fn buggy() { int x = 5; x++ return x; }' > target/ast-examples/errors.src
	@echo ""
	./target/$(TARGET)/debug/minic$(EXE) parse --input target/ast-examples/errors.src --show-metrics

ll1-demo:
	@echo "Демонстрация LL(1) анализа грамматики..."
	./target/$(TARGET)/debug/minic$(EXE) ll1 --show-first --show-follow

full-pipeline:
	@echo "Демонстрация полного пайплайна..."
	@mkdir -p target/ast-examples
	@echo '#define MAX 100' > target/ast-examples/full.src
	@echo 'fn main() -> int { int sum = 0; for (int i = 0; i < MAX; i++) { sum = sum + i; } return sum; }' >> target/ast-examples/full.src
	@echo ""
	./target/$(TARGET)/debug/minic$(EXE) full --input target/ast-examples/full.src --show-metrics

# === НОВЫЕ ДЕМОНСТРАЦИИ SPRINT 6 ===

sprint6-demo:
	@echo "=============================================="
	@echo "  ДЕМОНСТРАЦИЯ ВОЗМОЖНОСТЕЙ SPRINT 6"
	@echo "=============================================="
	@echo ""
	@$(MAKE) switch-demo
	@echo ""
	@$(MAKE) break-continue-demo
	@echo ""
	@$(MAKE) short-circuit-demo
	@echo ""
	@$(MAKE) float-demo
	@echo ""
	@$(MAKE) array-demo
	@echo ""
	@echo "=============================================="
	@echo "  ВСЕ ДЕМОНСТРАЦИИ SPRINT 6 ЗАВЕРШЕНЫ"
	@echo "=============================================="

switch-demo:
	@echo "----------------------------------------------"
	@echo "  ДЕМОНСТРАЦИЯ: Switch-case-default"
	@echo "----------------------------------------------"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int x = 2; int result = 0; switch (x) { case 1: result = 10; case 2: result = 20; case 3: result = 30; default: result = 0; } return result; }' > target/sprint6-examples/switch.src
	@echo "Исходный код:"
	@cat target/sprint6-examples/switch.src
	@echo ""
	@echo "AST (DOT):"
	./target/$(TARGET)/debug/minic$(EXE) parse --input target/sprint6-examples/switch.src --ast-format dot --output target/sprint6-examples/switch.dot
	@echo "  DOT сохранен в: target/sprint6-examples/switch.dot"
	@echo ""
	@echo "Генерация x86-64 кода:"
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/sprint6-examples/switch.src --output target/sprint6-examples/switch.asm
	@cat target/sprint6-examples/switch.asm

break-continue-demo:
	@echo "----------------------------------------------"
	@echo "  ДЕМОНСТРАЦИЯ: Break и Continue"
	@echo "----------------------------------------------"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int i = 0; int sum = 0; while (true) { i = i + 1; if (i > 10) { break; } if (i % 2 == 0) { continue; } sum = sum + i; } return sum; }' > target/sprint6-examples/break_continue.src
	@echo "Исходный код (сумма нечетных 1+3+5+7+9 = 25):"
	@cat target/sprint6-examples/break_continue.src
	@echo ""
	@echo "IR:"
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/sprint6-examples/break_continue.src --ir-format text
	@echo ""
	@echo "Генерация x86-64 кода:"
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/sprint6-examples/break_continue.src --output target/sprint6-examples/break_continue.asm --stats
	@echo ""
	@echo "Компиляция и запуск:"
	nasm -f elf64 target/sprint6-examples/break_continue.asm -o target/sprint6-examples/break_continue.o
	ld target/sprint6-examples/break_continue.o -o target/sprint6-examples/break_continue$(EXE)
	./target/sprint6-examples/break_continue$(EXE); echo "Exit: $$?"

short-circuit-demo:
	@echo "----------------------------------------------"
	@echo "  ДЕМОНСТРАЦИЯ: Короткая схема (Short-Circuit)"
	@echo "----------------------------------------------"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int a = 0; int b = 5; if (a != 0 && b / a > 2) { return 1; } return 0; }' > target/sprint6-examples/short_circuit.src
	@echo "Исходный код (деление на ноль предотвращено короткой схемой):"
	@cat target/sprint6-examples/short_circuit.src
	@echo ""
	@echo "IR:"
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/sprint6-examples/short_circuit.src --ir-format text
	@echo ""
	@echo "Генерация x86-64 кода:"
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/sprint6-examples/short_circuit.src --output target/sprint6-examples/short_circuit.asm
	@echo ""
	@echo "Компиляция и запуск (должен вернуть 0, а не упасть с ошибкой деления):"
	nasm -f elf64 target/sprint6-examples/short_circuit.asm -o target/sprint6-examples/short_circuit.o
	ld target/sprint6-examples/short_circuit.o -o target/sprint6-examples/short_circuit$(EXE)
	./target/sprint6-examples/short_circuit$(EXE); echo "Exit: $$?"

float-demo:
	@echo "----------------------------------------------"
	@echo "  ДЕМОНСТРАЦИЯ: Float и приведение типов"
	@echo "----------------------------------------------"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int x = 5; float y = 3.14; float z = x + y; return 15; }' > target/sprint6-examples/float.src
	@echo "Исходный код (int + float = float):"
	@cat target/sprint6-examples/float.src
	@echo ""
	@echo "IR:"
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/sprint6-examples/float.src --ir-format text
	@echo ""
	@echo "Генерация x86-64 кода:"
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/sprint6-examples/float.src --output target/sprint6-examples/float.asm
	@cat target/sprint6-examples/float.asm

array-demo:
	@echo "----------------------------------------------"
	@echo "  ДЕМОНСТРАЦИЯ: Массивы"
	@echo "----------------------------------------------"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int arr[3]; arr[0] = 10; arr[1] = 20; arr[2] = 30; return arr[1]; }' > target/sprint6-examples/array.src
	@echo "Исходный код (возвращает arr[1] = 20):"
	@cat target/sprint6-examples/array.src
	@echo ""
	@echo "IR:"
	./target/$(TARGET)/debug/minic$(EXE) ir --input target/sprint6-examples/array.src --ir-format text
	@echo ""
	@echo "Генерация x86-64 кода:"
	./target/$(TARGET)/debug/minic$(EXE) codegen --input target/sprint6-examples/array.src --output target/sprint6-examples/array.asm
	@cat target/sprint6-examples/array.asm

# === Примеры использования ===
example:
	@echo "Примеры использования компилятора:"
	@echo ""
	@echo "Генерация ассемблера:"
	@echo "  ./target/$(TARGET)/debug/minic$(EXE) codegen --input examples/factorial.src --output factorial.asm"
	@echo "  ./target/$(TARGET)/debug/minic$(EXE) codegen --input examples/factorial.src --optimize --stats"
	@echo "  ./target/$(TARGET)/debug/minic$(EXE) codegen --input examples/loop.src --output loop.asm"
	@echo ""
	@echo "Сборка и запуск сгенерированного кода:"
	@echo "  nasm -f elf64 factorial.asm -o factorial.o"
	@echo "  gcc -no-pie -o factorial.exe factorial.o"
	@echo "  ./factorial.exe"
	@echo ""
	@echo "Демонстрация Sprint 6:"
	@echo "  make sprint6-demo"

create-test-files:
	@echo "Создание тестовых файлов..."
	@mkdir -p examples tests/ir/golden tests/ir/golden/expected
	@echo 'fn main() -> int { return 42; }' > examples/hello.src
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } return n * factorial(n - 1); }' > examples/factorial.src
	@echo 'fn main() -> int { int sum = 0; int i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } return sum; }' > examples/loop.src
	@echo "Тестовые файлы созданы"

# === Утилиты ===
graphviz-check:
	@echo "Проверка наличия Graphviz..."
	@command -v dot >/dev/null 2>&1 && echo "Graphviz установлен" || echo "Graphviz не установлен"

install-deps:
	@echo "Установка зависимостей для разработки..."
	@cargo install cargo-udeps || true
	@cargo install cargo-tarpaulin || true

# === Справка ===
help:
	@echo "Mini Compiler - Sprint 6: Control Flow & Short-Circuit Evaluation"
	@echo ""
	@echo "Основные команды:"
	@echo "  make build         - Сборка проекта"
	@echo "  make release       - Сборка в режиме релиза"
	@echo "  make check         - Проверка кода"
	@echo "  make clean         - Очистка"
	@echo ""
	@echo "Тестирование:"
	@echo "  make test               - Запуск всех тестов"
	@echo "  make test-unit          - Unit тесты"
	@echo "  make test-codegen       - Тесты кодогенерации"
	@echo "  make test-control-flow  - Тесты потока управления (НОВОЕ!)"
	@echo "  make test-abi           - ABI compliance тесты"
	@echo "  make test-all           - Все тесты"
	@echo ""
	@echo "Демонстрации Sprint 6:"
	@echo "  make sprint6-demo       - Полная демонстрация всех новых возможностей"
	@echo "  make switch-demo        - Switch-case-default"
	@echo "  make break-continue-demo- Break и Continue"
	@echo "  make short-circuit-demo - Короткая схема вычислений"
	@echo "  make float-demo         - Float и приведение типов"
	@echo "  make array-demo         - Массивы"
	@echo ""
	@echo "Демонстрации:"
	@echo "  make codegen-demo  - Демонстрация кодогенерации"
	@echo "  make ir-demo       - Демонстрация IR"
	@echo "  make ast-demo      - Демонстрация AST"
	@echo "  make optimization-demo - Оптимизации IR"
	@echo ""
	@echo "Качество кода:"
	@echo "  make lint          - Проверка линтером"
	@echo "  make fmt           - Форматирование кода"
	@echo ""
	@echo "Документация:"
	@echo "  make docs          - Генерация документации"
	@echo ""
	@echo "Быстрый старт:"
	@echo "  make build && make sprint6-demo"