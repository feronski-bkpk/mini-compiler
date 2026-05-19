.PHONY: all build release check test test-lexer test-parser test-preprocessor test-semantic \
        test-ir test-ir-opt test-ir-golden test-integration test-ll1 test-all test-codegen \
        test-codegen-unit test-codegen-integration test-abi test-register clean docs help \
        ast-demo ir-demo semantic-demo var-demo inc-demo error-demo ll1-demo optimization-demo \
        codegen-demo full-pipeline sprint6-demo sprint7-demo switch-demo break-continue-demo short-circuit-demo \
        float-demo array-demo inline-demo lint fmt fmt-check udeps bench coverage graphviz-check \
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
	$(CARGO) test $(CARGO_FLAGS) --test integration_codegen -- --nocapture
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
	@$(call RMRF,target/sprint7-examples)
	@echo "Удаление сгенерированных файлов из examples/..."
	@-$(RM) examples/*.asm 2>/dev/null
	@-$(RM) examples/*.o 2>/dev/null
	@-$(RM) examples/*.exe 2>/dev/null
	@-$(RM) examples/*.json 2>/dev/null
	@-$(RM) examples/*.dot 2>/dev/null
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
	./target/debug/minic parse --input target/ast-examples/simple.src
	@echo ""
	@echo "Текстовый формат AST (с инкрементами):"
	./target/debug/minic parse --input target/ast-examples/increment.src
	@echo ""
	@echo "Генерация DOT графа..."
	./target/debug/minic parse --input target/ast-examples/struct.src --ast-format dot --output target/ast-examples/ast.dot
	@echo "  DOT файл сохранен: target/ast-examples/ast.dot"

ir-demo:
	@echo "Демонстрация генерации промежуточного представления (IR)..."
	@mkdir -p target/ir-examples
	@echo 'fn main() -> int { int x = 5; int y = 10; int z = x + y; return z; }' > target/ir-examples/simple.src
	@echo 'fn main() -> int { int x = 5; if (x > 0) { return 10; } else { return 20; } }' > target/ir-examples/if.src
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } }' > target/ir-examples/factorial.src
	@echo ""
	@echo "=== Текстовый формат IR (простая арифметика) ==="
	./target/debug/minic ir --input target/ir-examples/simple.src --ir-format text
	@echo ""
	@echo "=== Текстовый формат IR (if-else) ==="
	./target/debug/minic ir --input target/ir-examples/if.src --ir-format text
	@echo ""
	@echo "=== Текстовый формат IR (рекурсивный факториал) ==="
	./target/debug/minic ir --input target/ir-examples/factorial.src --ir-format text
	@echo ""
	@echo "=== Статистика IR ==="
	./target/debug/minic ir --input target/ir-examples/factorial.src --stats
	@echo ""
	@echo "=== Генерация DOT графа потока управления (CFG) ==="
	./target/debug/minic ir --input target/ir-examples/if.src --ir-format dot --output target/ir-examples/cfg.dot
	@echo "  DOT файл сохранен: target/ir-examples/cfg.dot"

codegen-demo:
	@echo "Демонстрация генерации x86-64 кода..."
	@mkdir -p target/codegen-examples
	@echo 'fn main() -> int { return 42; }' > target/codegen-examples/simple.src
	@echo 'fn add(int a, int b) -> int { return a + b; } fn main() -> int { return add(5, 3); }' > target/codegen-examples/call.src
	@echo 'fn main() -> int { int sum = 0; int i = 1; while (i <= 10) { sum = sum + i; i = i + 1; } return sum; }' > target/codegen-examples/loop.src
	@echo ""
	@echo "=== Генерация ассемблера (простая функция) ==="
	./target/debug/minic codegen --input target/codegen-examples/simple.src --output target/codegen-examples/simple.asm
	@cat target/codegen-examples/simple.asm
	@echo ""
	@echo "=== Генерация ассемблера (вызов функции) ==="
	./target/debug/minic codegen --input target/codegen-examples/call.src --output target/codegen-examples/call.asm
	@echo ""
	@echo "=== Генерация ассемблера (цикл) ==="
	./target/debug/minic codegen --input target/codegen-examples/loop.src --output target/codegen-examples/loop.asm
	@echo ""
	@echo "=== Статистика кодогенерации ==="
	./target/debug/minic codegen --input target/codegen-examples/loop.src --output target/codegen-examples/loop.asm --stats

optimization-demo:
	@echo "Демонстрация оптимизаций IR..."
	@mkdir -p target/ir-examples
	@echo 'fn main() -> int { int x = 2 + 3; int y = 5 * 4; int z = 10 - 3; return x + y + z; }' > target/ir-examples/const_fold.src
	@echo 'fn main() -> int { int x = 5; int a = x + 0; int b = x * 1; int c = x * 0; return a + b + c; }' > target/ir-examples/algebraic.src
	@echo 'fn main() -> int { int x = 5; int y = 10; int z = x + y; int w = z * 2; return x; }' > target/ir-examples/dead_code.src
	@echo ""
	@echo "=== Свертка констант (до оптимизации) ==="
	./target/debug/minic ir --input target/ir-examples/const_fold.src --ir-format text
	@echo ""
	@echo "=== Свертка констант (после оптимизации) ==="
	./target/debug/minic ir --input target/ir-examples/const_fold.src --ir-format text --optimize --verbose
	@echo ""
	@echo "=== Алгебраические упрощения (до) ==="
	./target/debug/minic ir --input target/ir-examples/algebraic.src --ir-format text
	@echo ""
	@echo "=== Алгебраические упрощения (после) ==="
	./target/debug/minic ir --input target/ir-examples/algebraic.src --ir-format text --optimize --verbose
	@echo ""
	@echo "=== Удаление мертвого кода (до) ==="
	./target/debug/minic ir --input target/ir-examples/dead_code.src --ir-format text
	@echo ""
	@echo "=== Удаление мертвого кода (после) ==="
	./target/debug/minic ir --input target/ir-examples/dead_code.src --ir-format text --optimize --verbose

inline-demo:
	@echo "Демонстрация Function Inlining (Stretch Goal)..."
	@mkdir -p target/sprint7-examples
	@echo 'int square(int x) { return x * x; }' > target/sprint7-examples/inline.src
	@echo 'int cube(int x) { return x * x * x; }' >> target/sprint7-examples/inline.src
	@echo 'int main() { int a = square(5); int b = cube(3); return a + b; }' >> target/sprint7-examples/inline.src
	@echo ""
	@echo "=== IR ДО инлайнинга ==="
	./target/debug/minic ir --input target/sprint7-examples/inline.src --ir-format text
	@echo ""
	@echo "=== IR ПОСЛЕ инлайнинга ==="
	./target/debug/minic ir --input target/sprint7-examples/inline.src --inline --ir-format text
	@echo ""
	@echo "=== Компиляция с инлайнингом и оптимизациями ==="
	./target/debug/minic codegen --input target/sprint7-examples/inline.src --output target/sprint7-examples/inline.asm --inline --optimize --stats
	@echo ""
	@echo "=== Запуск (square(5)=25, cube(3)=27, 25+27=52) ==="
	nasm -f elf64 target/sprint7-examples/inline.asm -o target/sprint7-examples/inline.o
	gcc -no-pie target/sprint7-examples/inline.o -o target/sprint7-examples/inline
	./target/sprint7-examples/inline; echo "Exit: $$?"

semantic-demo:
	@echo "Демонстрация семантического анализа..."
	@mkdir -p target/ast-examples
	@echo 'fn add(int a, int b) -> int { return a + b; }' > target/ast-examples/correct.src
	@echo 'fn main() -> int { int x = y + 5; return x; }' > target/ast-examples/undeclared.src
	@echo 'fn main() -> int { int x = 3.14; return x; }' > target/ast-examples/type_mismatch.src
	@echo ""
	@echo "=== Корректная программа ==="
	./target/debug/minic semantic --input target/ast-examples/correct.src --show-symbols
	@echo ""
	@echo "=== Необъявленная переменная ==="
	./target/debug/minic semantic --input target/ast-examples/undeclared.src || true

var-demo:
	@echo "Демонстрация вывода типов (var)..."
	@mkdir -p target/ast-examples
	@echo 'fn main() -> int { var x = 42; var y = 3.14; var z = true; var s = "hello"; return 0; }' > target/ast-examples/var_demo.src
	@echo ""
	./target/debug/minic semantic --input target/ast-examples/var_demo.src --show-symbols

inc-demo:
	@echo "Демонстрация инкрементов/декрементов..."
	@mkdir -p target/ast-examples
	@echo 'fn main() -> int { int x = 5; int a = x++; int b = ++x; int c = x--; int d = --x; return x; }' > target/ast-examples/inc.src
	@echo ""
	./target/debug/minic parse --input target/ast-examples/inc.src

error-demo:
	@echo "Демонстрация восстановления после ошибок..."
	@mkdir -p target/ast-examples
	@echo 'fn buggy() { int x = 5; x++ return x; }' > target/ast-examples/errors.src
	@echo ""
	./target/debug/minic parse --input target/ast-examples/errors.src --show-metrics

ll1-demo:
	@echo "Демонстрация LL(1) анализа грамматики..."
	./target/debug/minic ll1 --show-first --show-follow

full-pipeline:
	@echo "Демонстрация полного пайплайна..."
	@mkdir -p target/ast-examples
	@echo '#define MAX 100' > target/ast-examples/full.src
	@echo 'fn main() -> int { int sum = 0; for (int i = 0; i < MAX; i++) { sum = sum + i; } return sum; }' >> target/ast-examples/full.src
	@echo ""
	./target/debug/minic full --input target/ast-examples/full.src --show-metrics

# === ДЕМОНСТРАЦИИ SPRINT 6 ===

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
	@echo "--- Switch-case-default ---"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int x = 2; int result = 0; switch (x) { case 1: result = 10; case 2: result = 20; case 3: result = 30; default: result = 0; } return result; }' > target/sprint6-examples/switch.src
	./target/debug/minic codegen --input target/sprint6-examples/switch.src --output target/sprint6-examples/switch.asm --stats
	@cat target/sprint6-examples/switch.asm

break-continue-demo:
	@echo "--- Break и Continue ---"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int sum = 0; int i = 0; while (true) { i = i + 1; if (i > 10) { break; } if (i % 2 == 0) { continue; } sum = sum + i; } return sum; }' > target/sprint6-examples/break_continue.src
	./target/debug/minic codegen --input target/sprint6-examples/break_continue.src --output target/sprint6-examples/break_continue.asm --stats

short-circuit-demo:
	@echo "--- Короткая схема (Short-Circuit) ---"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int a = 0; int b = 5; if (a != 0 && b / a > 2) { return 1; } return 0; }' > target/sprint6-examples/short_circuit.src
	./target/debug/minic codegen --input target/sprint6-examples/short_circuit.src --output target/sprint6-examples/short_circuit.asm

float-demo:
	@echo "--- Float и приведение типов ---"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int x = 5; float y = 3.14; float z = x + y; return 15; }' > target/sprint6-examples/float.src
	./target/debug/minic codegen --input target/sprint6-examples/float.src --output target/sprint6-examples/float.asm

array-demo:
	@echo "--- Массивы ---"
	@mkdir -p target/sprint6-examples
	@echo 'fn main() -> int { int arr[3]; arr[0] = 10; arr[1] = 20; arr[2] = 30; return arr[1]; }' > target/sprint6-examples/array.src
	./target/debug/minic codegen --input target/sprint6-examples/array.src --output target/sprint6-examples/array.asm

# === ДЕМОНСТРАЦИЯ SPRINT 7 ===

sprint7-demo:
	@echo "=============================================="
	@echo "  ДЕМОНСТРАЦИЯ ВОЗМОЖНОСТЕЙ SPRINT 7"
	@echo "=============================================="
	@echo ""
	@echo "=== Sprint 7 Features ==="
	@echo "  - External functions (printf from libc)"
	@echo "  - Variadic arguments"
	@echo "  - Stack arrays with initialization"
	@echo "  - Arrays as function parameters"
	@echo "  - Pointers (* and &)"
	@echo "  - Recursive functions"
	@echo "  - Function inlining (stretch goal)"
	@echo "  - Peephole optimizations"
	@echo ""
	@mkdir -p target/sprint7-examples
	@cp examples/sprint7_full_demo.src target/sprint7-examples/demo.src 2>/dev/null || \
		echo "Creating demo file..."
	@echo ""
	@echo "=== Компиляция с инлайнингом и оптимизациями ==="
	./target/debug/minic codegen --input examples/sprint7_full_demo.src --output target/sprint7-examples/demo.asm --inline --optimize --stats 2>/dev/null || \
	./target/debug/minic codegen --input target/sprint7-examples/demo.src --output target/sprint7-examples/demo.asm --inline --optimize --stats
	@echo ""
	@echo "=== Сборка и запуск ==="
	nasm -f elf64 target/sprint7-examples/demo.asm -o target/sprint7-examples/demo.o
	gcc -no-pie target/sprint7-examples/demo.o -o target/sprint7-examples/demo
	./target/sprint7-examples/demo
	@echo ""
	@echo "=============================================="
	@echo "  ДЕМОНСТРАЦИЯ SPRINT 7 ЗАВЕРШЕНА"
	@echo "=============================================="

# === Примеры использования ===
example:
	@echo "Примеры использования компилятора:"
	@echo ""
	@echo "Генерация ассемблера:"
	@echo "  ./target/debug/minic codegen --input examples/factorial.src --output factorial.asm"
	@echo "  ./target/debug/minic codegen --input examples/factorial.src --optimize --stats"
	@echo "  ./target/debug/minic codegen --input examples/loop.src --output loop.asm"
	@echo ""
	@echo "Сборка и запуск сгенерированного кода:"
	@echo "  nasm -f elf64 factorial.asm -o factorial.o"
	@echo "  gcc -no-pie -o factorial factorial.o"
	@echo "  ./factorial"
	@echo ""
	@echo "Демонстрация Sprint 7:"
	@echo "  make sprint7-demo"
	@echo ""
	@echo "Демонстрация инлайнинга:"
	@echo "  make inline-demo"

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
	@echo "Mini Compiler - Sprint 7: Arrays, Extern, Optimizations & Inlining"
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
	@echo "  make test-control-flow  - Тесты потока управления"
	@echo "  make test-abi           - ABI compliance тесты"
	@echo "  make test-all           - Все тесты"
	@echo ""
	@echo "Демонстрации Sprint 7:"
	@echo "  make sprint7-demo       - Полная демонстрация всех новых возможностей"
	@echo "  make inline-demo        - Демонстрация Function Inlining"
	@echo "  make optimization-demo   - Демонстрация оптимизаций"
	@echo ""
	@echo "Демонстрации Sprint 6:"
	@echo "  make sprint6-demo       - Полная демонстрация Sprint 6"
	@echo ""
	@echo "Демонстрации:"
	@echo "  make codegen-demo  - Демонстрация кодогенерации"
	@echo "  make ir-demo       - Демонстрация IR"
	@echo "  make ast-demo      - Демонстрация AST"
	@echo ""
	@echo "Качество кода:"
	@echo "  make lint          - Проверка линтером"
	@echo "  make fmt           - Форматирование кода"
	@echo ""
	@echo "Документация:"
	@echo "  make docs          - Генерация документации"
	@echo ""
	@echo "Быстрый старт:"
	@echo "  make build && make sprint7-demo"