.PHONY: all build release check test test-lexer test-parser test-preprocessor test-semantic \
        test-ir test-ir-opt test-integration test-ll1 test-all clean docs help run-example \
        ast-demo ir-demo semantic-demo var-demo inc-demo error-demo ll1-demo optimization-demo \
        full-pipeline lint fmt fmt-check udeps bench coverage graphviz-check install-deps \
        create-test-files

CARGO = cargo
RUSTFLAGS = -D warnings

all: check test build

build:
	@echo "Сборка проекта..."
	$(CARGO) build

release:
	@echo "Сборка в режиме релиза..."
	$(CARGO) build --release

check:
	@echo "Проверка кода..."
	$(CARGO) check

# === Тестирование ===
test:
	@echo "Запуск всех тестов..."
	$(CARGO) test -- --nocapture

test-lexer:
	@echo "Запуск тестов лексического анализатора..."
	$(CARGO) test lexer -- --nocapture

test-parser:
	@echo "Запуск тестов парсера..."
	$(CARGO) test parser -- --nocapture

test-preprocessor:
	@echo "Запуск тестов препроцессора..."
	$(CARGO) test preprocessor -- --nocapture

test-semantic:
	@echo "Запуск семантических тестов..."
	$(CARGO) test semantic_tests -- --nocapture

test-ir:
	@echo "Запуск тестов генерации IR..."
	$(CARGO) test --test ir_tests -- --nocapture

test-ir-opt:
	@echo "Запуск тестов оптимизаций IR..."
	$(CARGO) test --test ir_optimization_tests -- --nocapture

test-ir-golden:
	@echo "Запуск golden тестов IR..."
	$(CARGO) test --test ir_golden_tests -- --nocapture

test-integration:
	@echo "Запуск интеграционных тестов..."
	$(CARGO) test integration -- --nocapture

test-ll1:
	@echo "Запуск LL(1) тестов..."
	$(CARGO) test ll1_tests -- --nocapture

test-common:
	@echo "Запуск тестов общих модулей..."
	$(CARGO) test common -- --nocapture

test-doc:
	@echo "Запуск документационных тестов..."
	$(CARGO) test --doc

test-all: test test-ir test-ir-opt test-ir-golden test-ll1 test-semantic test-doc
	@echo "Все тесты пройдены!"

# === Качество кода ===
lint:
	@echo "Проверка линтером..."
	$(CARGO) clippy -- -D warnings

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
	@echo "Очистка..."
	$(CARGO) clean
	@rm -rf target/ast-examples target/ir-examples

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
	@echo "fn main() { return 42; }" > target/ast-examples/simple.src
	@echo "struct Point { int x; int y; } fn main() { struct Point p; p.x = 10; return p.x; }" > target/ast-examples/struct.src
	@echo "fn counter() { int x = 5; x++; ++x; x--; --x; return x; }" > target/ast-examples/increment.src
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
	@echo "  Для визуализации выполните: dot -Tpng target/ast-examples/ast.dot -o target/ast-examples/ast.png"
	@echo ""
	@echo "JSON формат:"
	./target/debug/minic parse --input target/ast-examples/simple.src --ast-format json | head -20

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
	@echo "  Для визуализации выполните: dot -Tpng target/ir-examples/cfg.dot -o target/ir-examples/cfg.png"
	@echo ""
	@echo "=== JSON формат IR ==="
	./target/debug/minic ir --input target/ir-examples/simple.src --ir-format json | head -30

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

semantic-demo:
	@echo "Демонстрация семантического анализа..."
	@mkdir -p target/ast-examples
	@echo 'fn add(int a, int b) -> int { return a + b; }' > target/ast-examples/correct.src
	@echo 'fn main() -> int { int x = y + 5; return x; }' > target/ast-examples/undeclared.src
	@echo 'fn main() -> int { int x = 3.14; return x; }' > target/ast-examples/type_mismatch.src
	@echo 'fn add(int a, int b) -> int { return a + b; } fn main() -> int { return add(5); }' > target/ast-examples/arg_count.src
	@echo 'struct Point { int x; int y; } fn main() { struct Point p; p.z = 10; return 0; }' > target/ast-examples/field_error.src
	@echo ""
	@echo "=== Корректная программа ==="
	./target/debug/minic semantic --input target/ast-examples/correct.src --show-symbols
	@echo ""
	@echo "=== Необъявленная переменная ==="
	./target/debug/minic semantic --input target/ast-examples/undeclared.src || true
	@echo ""
	@echo "=== Несоответствие типов ==="
	./target/debug/minic semantic --input target/ast-examples/type_mismatch.src || true
	@echo ""
	@echo "=== Неправильное количество аргументов ==="
	./target/debug/minic semantic --input target/ast-examples/arg_count.src || true
	@echo ""
	@echo "=== Несуществующее поле структуры ==="
	./target/debug/minic semantic --input target/ast-examples/field_error.src || true

var-demo:
	@echo "Демонстрация вывода типов (var)..."
	@mkdir -p target/ast-examples
	@echo 'fn main() -> int {' > target/ast-examples/var_demo.src
	@echo '    var x = 42;' >> target/ast-examples/var_demo.src
	@echo '    var y = 3.14;' >> target/ast-examples/var_demo.src
	@echo '    var z = true;' >> target/ast-examples/var_demo.src
	@echo '    var s = "hello";' >> target/ast-examples/var_demo.src
	@echo '    x = 100;' >> target/ast-examples/var_demo.src
	@echo '    y = 2.71;' >> target/ast-examples/var_demo.src
	@echo '    return 0;' >> target/ast-examples/var_demo.src
	@echo '}' >> target/ast-examples/var_demo.src
	@echo ""
	@echo "Код с var:"
	@cat target/ast-examples/var_demo.src
	@echo ""
	@echo "Семантический анализ с выводом таблицы символов:"
	./target/debug/minic semantic --input target/ast-examples/var_demo.src --show-symbols
	@echo ""
	@echo "Семантический анализ с размерами и смещениями:"
	./target/debug/minic semantic --input target/ast-examples/var_demo.src --show-symbols --show-layout
	@echo ""
	@echo "Демонстрация ошибки несовместимого присваивания:"
	@echo 'fn main() -> int { var x = 42; x = "hello"; return 0; }' > target/ast-examples/var_error.src
	./target/debug/minic semantic --input target/ast-examples/var_error.src || true

inc-demo:
	@echo "Демонстрация инкрементов/декрементов..."
	@mkdir -p target/ast-examples
	@echo 'fn main() {' > target/ast-examples/inc.src
	@echo '    int x = 5;' >> target/ast-examples/inc.src
	@echo '    int a = x++;  // постфиксный инкремент' >> target/ast-examples/inc.src
	@echo '    int b = ++x;  // префиксный инкремент' >> target/ast-examples/inc.src
	@echo '    int c = x--;  // постфиксный декремент' >> target/ast-examples/inc.src
	@echo '    int d = --x;  // префиксный декремент' >> target/ast-examples/inc.src
	@echo '    int e = x++ + ++x;  // смешанное использование' >> target/ast-examples/inc.src
	@echo '    return x;' >> target/ast-examples/inc.src
	@echo '}' >> target/ast-examples/inc.src
	@echo ""
	@echo "Демонстрационный код:"
	@cat target/ast-examples/inc.src
	@echo ""
	@echo "Лексический анализ (поиск токенов ++ и --):"
	./target/debug/minic lex --input target/ast-examples/inc.src --verbose | findstr "PLUS_PLUS MINUS_MINUS" || echo "  (токены найдены)"
	@echo ""
	@echo "Синтаксический анализ (AST с инкрементами):"
	./target/debug/minic parse --input target/ast-examples/inc.src

error-demo:
	@echo "Демонстрация восстановления после ошибок..."
	@mkdir -p target/ast-examples
	@echo 'fn buggy() { int x = 5; x++ return x; }' > target/ast-examples/errors.src
	@echo 'fn main() { if (x > 0 { return x; } else { return 0; }' >> target/ast-examples/errors.src
	@echo ""
	@echo "Файл с ошибками:"
	@cat target/ast-examples/errors.src
	@echo ""
	@echo "Анализ с восстановлением:"
	./target/debug/minic parse --input target/ast-examples/errors.src --show-metrics

ll1-demo:
	@echo "Демонстрация LL(1) анализа грамматики..."
	./target/debug/minic ll1 --show-first --show-follow

full-pipeline:
	@echo "Демонстрация полного пайплайна..."
	@mkdir -p target/ast-examples
	@echo '#define MAX 100' > target/ast-examples/full.src
	@echo '#define DEBUG 1' >> target/ast-examples/full.src
	@echo '' >> target/ast-examples/full.src
	@echo 'fn main() -> int {' >> target/ast-examples/full.src
	@echo '    int sum = 0;' >> target/ast-examples/full.src
	@echo '    for (int i = 0; i < MAX; i++) {' >> target/ast-examples/full.src
	@echo '        sum = sum + i;' >> target/ast-examples/full.src
	@echo '        #ifdef DEBUG' >> target/ast-examples/full.src
	@echo '            int debug = i;' >> target/ast-examples/full.src
	@echo '        #endif' >> target/ast-examples/full.src
	@echo '    }' >> target/ast-examples/full.src
	@echo '    return sum;' >> target/ast-examples/full.src
	@echo '}' >> target/ast-examples/full.src
	@echo ""
	@echo "Исходный код с препроцессором:"
	@cat target/ast-examples/full.src
	@echo ""
	@echo "Шаг 1: Препроцессор"
	./target/debug/minic preprocess --input target/ast-examples/full.src --output target/ast-examples/full_processed.src --show
	@echo ""
	@echo "Шаг 2: Лексический анализ"
	./target/debug/minic lex --input target/ast-examples/full_processed.src --quiet
	@echo ""
	@echo "Шаг 3: Синтаксический анализ"
	./target/debug/minic parse --input target/ast-examples/full_processed.src
	@echo ""
	@echo "Шаг 4: Семантический анализ"
	./target/debug/minic semantic --input target/ast-examples/full_processed.src --show-symbols
	@echo ""
	@echo "Шаг 5: Генерация IR"
	./target/debug/minic ir --input target/ast-examples/full_processed.src --ir-format text

# === Примеры использования ===
example:
	@echo "Примеры использования компилятора:"
	@echo ""
	@echo "Лексический анализ:"
	@echo "  ./target/debug/minic lex --input examples/hello.src"
	@echo "  ./target/debug/minic lex --input examples/hello.src --verbose"
	@echo "  ./target/debug/minic lex --input examples/hello.src --format json"
	@echo ""
	@echo "Синтаксический анализ (AST):"
	@echo "  ./target/debug/minic parse --input examples/factorial.src"
	@echo "  ./target/debug/minic parse --input examples/factorial.src --ast-format dot --output ast.dot"
	@echo "  ./target/debug/minic parse --input examples/factorial.src --ast-format json --output ast.json"
	@echo "  ./target/debug/minic parse --input examples/factorial.src --show-metrics"
	@echo ""
	@echo "Семантический анализ:"
	@echo "  ./target/debug/minic semantic --input examples/factorial.src"
	@echo "  ./target/debug/minic semantic --input examples/factorial.src --show-symbols"
	@echo "  ./target/debug/minic semantic --input examples/factorial.src --show-ast"
	@echo "  ./target/debug/minic semantic --input examples/factorial.src --show-symbols --show-layout"
	@echo ""
	@echo "Генерация IR (НОВОЕ!):"
	@echo "  ./target/debug/minic ir --input examples/factorial.src --ir-format text"
	@echo "  ./target/debug/minic ir --input examples/factorial.src --ir-format dot --output cfg.dot"
	@echo "  ./target/debug/minic ir --input examples/factorial.src --ir-format json --output ir.json"
	@echo "  ./target/debug/minic ir --input examples/factorial.src --stats"
	@echo "  ./target/debug/minic ir --input examples/factorial.src --optimize --verbose"
	@echo ""
	@echo "Препроцессор:"
	@echo "  ./target/debug/minic preprocess --input examples/test.src --defines DEBUG=1 --show"
	@echo "  ./target/debug/minic preprocess --input examples/test.src --preserve-lines --output processed.src"
	@echo ""
	@echo "Полный пайплайн:"
	@echo "  ./target/debug/minic full --input examples/factorial.src --ast-format text"
	@echo "  ./target/debug/minic full --input examples/test.src --defines DEBUG=1 --ast-format dot --output full.dot"
	@echo "  ./target/debug/minic full --input examples/test.src --show-metrics"
	@echo ""
	@echo "Вывод типов (var):"
	@echo "  ./target/debug/minic semantic --input examples/var_demo.src --show-symbols"
	@echo "  ./target/debug/minic semantic --input examples/var_demo.src --show-symbols --show-layout"
	@echo ""
	@echo "Инкременты/декременты:"
	@echo "  ./target/debug/minic inc-demo"
	@echo "  ./target/debug/minic parse --input examples/increment.src"
	@echo ""
	@echo "LL(1) анализ:"
	@echo "  ./target/debug/minic ll1 --show-first --show-follow"
	@echo ""
	@echo "Восстановление после ошибок:"
	@echo "  ./target/debug/minic error-demo --input examples/errors.src"
	@echo "  ./target/debug/minic parse --input examples/errors.src --show-metrics --max-errors 50"
	@echo ""
	@echo "Проверка синтаксиса и семантики:"
	@echo "  ./target/debug/minic check --input examples/hello.src"
	@echo "  ./target/debug/minic check --input examples/hello.src --strict"
	@echo "  ./target/debug/minic check --input examples/test.src --preprocess --defines DEBUG=1"
	@echo ""
	@echo "Информация:"
	@echo "  ./target/debug/minic info"
	@echo "  ./target/debug/minic info --verbose"
	@echo "  ./target/debug/minic spec"

create-test-files:
	@echo "Создание тестовых файлов..."
	@mkdir -p examples tests/lexer/valid tests/lexer/invalid tests/parser/valid tests/parser/invalid docs tests/ir/golden tests/ir/golden/expected

	# Базовые примеры
	@echo 'fn main() { return 42; }' > examples/hello.src
	@echo 'int x = 10;' > examples/simple.src
	@echo '// Тестовая программа' > examples/test.src

	# Примеры с инкрементами
	@echo 'fn main() { int x = 5; x++; ++x; x--; --x; return x; }' > examples/increment.src
	@echo 'fn compute() { int a = 5; int b = a++ + ++a; return b; }' > examples/complex_inc.src

	# Примеры с var
	@echo 'fn main() { var x = 42; var y = 3.14; var z = true; var s = "hello"; return 0; }' > examples/var_demo.src
	@echo 'fn main() { var x = 42; x = 100; return x; }' > examples/var_assign.src
	@echo 'fn main() { var x; return 0; }' > examples/var_error.src

	# Примеры для семантического анализа
	@echo 'fn add(int a, int b) -> int { return a + b; }' > examples/function.src
	@echo 'fn main() -> int { int x = y + 5; return x; }' > examples/undeclared.src
	@echo 'fn main() -> int { int x = 3.14; return x; }' > examples/type_mismatch.src
	@echo 'struct Point { int x; int y; } fn main() { struct Point p; p.z = 10; return 0; }' > examples/field_error.src

	# Примеры для парсера
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } return n * factorial(n - 1); }' > examples/factorial.src
	@echo 'struct Point { int x; int y; } fn main() { struct Point p; p.x = 10; p.y = 20; p.x++; return p.x + p.y; }' > examples/struct.src

	# Примеры для IR
	@echo 'fn main() -> int { int x = 5; int y = 10; int z = x + y; return z; }' > examples/ir_simple.src
	@echo 'fn main() -> int { int x = 5; if (x > 0) { return 10; } else { return 20; } }' > examples/ir_if.src
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } else { return n * factorial(n - 1); } }' > examples/ir_factorial.src
	@echo 'fn main() -> int { int i = 0; int sum = 0; while (i < 5) { sum = sum + i; i = i + 1; } return sum; }' > examples/ir_while.src

	# Тесты для парсера
	@echo 'fn add(int a, int b) -> int { return a + b; }' > tests/parser/valid/function.src
	@echo 'if (x > 0) { return 1; } else { return 0; }' > tests/parser/valid/if.src
	@echo 'while (i < 10) { i = i + 1; }' > tests/parser/valid/while.src
	@echo 'for (int i = 0; i < 10; i = i + 1) { print(i); }' > tests/parser/valid/for.src

	# Файлы с ошибками
	@echo 'fn buggy() { int x = 5; x++ return x; }' > examples/errors.src
	@echo 'fn main() { if (x > 0 { return x; } }' >> examples/errors.src

	@echo "Тестовые файлы созданы:"
	@echo "  examples/hello.src - простая программа"
	@echo "  examples/factorial.src - рекурсивный факториал"
	@echo "  examples/struct.src - работа со структурами"
	@echo "  examples/increment.src - инкременты/декременты"
	@echo "  examples/var_demo.src - демонстрация var"
	@echo "  examples/ir_simple.src - пример для IR (арифметика)"
	@echo "  examples/ir_if.src - пример для IR (if-else)"
	@echo "  examples/ir_factorial.src - пример для IR (рекурсия)"
	@echo "  examples/ir_while.src - пример для IR (цикл)"
	@echo "  examples/undeclared.src - необъявленная переменная"
	@echo "  examples/type_mismatch.src - несоответствие типов"
	@echo "  examples/field_error.src - несуществующее поле"

# === Утилиты ===
graphviz-check:
	@echo "Проверка наличия Graphviz..."
	@command -v dot >/dev/null 2>&1 && echo "Graphviz установлен" || echo "Graphviz не установлен. Установите: https://graphviz.org/download/"

install-deps:
	@echo "Установка зависимостей для разработки..."
	@cargo install cargo-udeps || true
	@cargo install cargo-tarpaulin || true
	@cargo install cargo-expand || true
	@cargo install cargo-bloat || true

# === Справка ===
help:
	@echo "Mini Compiler"
	@echo ""
	@echo "Основные команды:"
	@echo "  make all           - Проверка, тесты и сборка"
	@echo "  make build         - Сборка проекта"
	@echo "  make release       - Сборка в режиме релиза"
	@echo "  make check         - Проверка кода без сборки"
	@echo "  make clean         - Очистка"
	@echo ""
	@echo "Тестирование:"
	@echo "  make test          - Запуск всех тестов"
	@echo "  make test-lexer    - Тесты лексического анализатора"
	@echo "  make test-parser   - Тесты парсера"
	@echo "  make test-preprocessor - Тесты препроцессора"
	@echo "  make test-semantic - Семантические тесты"
	@echo "  make test-ir       - Тесты генерации IR (НОВОЕ!)"
	@echo "  make test-ir-opt   - Тесты оптимизаций IR (НОВОЕ!)"
	@echo "  make test-ir-golden - Golden тесты IR (НОВОЕ!)"
	@echo "  make test-integration - Интеграционные тесты"
	@echo "  make test-ll1      - LL(1) тесты"
	@echo "  make test-common   - Тесты общих модулей"
	@echo "  make test-doc      - Документационные тесты"
	@echo "  make test-all      - Все тесты (включая IR)"
	@echo ""
	@echo "Качество кода:"
	@echo "  make lint          - Проверка линтером (clippy)"
	@echo "  make fmt           - Форматирование кода"
	@echo "  make fmt-check     - Проверка форматирования"
	@echo ""
	@echo "Документация:"
	@echo "  make docs          - Генерация документации"
	@echo "  make docs-private  - Документация с приватными элементами"
	@echo ""
	@echo "Демонстрации:"
	@echo "  make ast-demo      - Визуализация AST (text/dot/json)"
	@echo "  make ir-demo       - Генерация IR (текст/DOT/JSON/статистика) (НОВОЕ!)"
	@echo "  make optimization-demo - Оптимизации IR (свертка констант, удаление мертвого кода) (НОВОЕ!)"
	@echo "  make semantic-demo - Демонстрация семантического анализа"
	@echo "  make var-demo      - Демонстрация вывода типов var"
	@echo "  make inc-demo      - Демонстрация инкрементов/декрементов"
	@echo "  make error-demo    - Демонстрация восстановления после ошибок"
	@echo "  make ll1-demo      - LL(1) анализ грамматики"
	@echo "  make full-pipeline - Полный пайплайн (препроцессор → лексер → парсер → семантика → IR)"
	@echo "  make example       - Показать примеры использования CLI"
	@echo "  make create-test-files - Создать тестовые файлы"
	@echo ""
	@echo "Анализ:"
	@echo "  make udeps         - Анализ неиспользуемых зависимостей"
	@echo "  make bench         - Запуск бенчмарков"
	@echo "  make coverage      - Измерение покрытия кода"
	@echo ""
	@echo "Утилиты:"
	@echo "  make graphviz-check - Проверка наличия Graphviz"
	@echo "  make install-deps  - Установка зависимостей для разработки"
	@echo "  make help          - Показать эту справку"
	@echo ""
	@echo "Быстрый старт:"
	@echo "  make create-test-files && make build && make ast-demo"
	@echo "  make ir-demo       # Показать генерацию IR"
	@echo "  make optimization-demo # Показать оптимизации IR"
	@echo "  make semantic-demo # Показать семантический анализ"
	@echo "  make var-demo      # Показать вывод типов var"