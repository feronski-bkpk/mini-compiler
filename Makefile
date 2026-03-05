.PHONY: all build release check test test-parser test-preprocessor test-integration test-ll1 \
        test-all clean docs help run-example ast-demo error-demo inc-demo ll1-demo \
        lint fmt fmt-check udeps bench coverage graphviz-check install-deps

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

test-all: test test-ll1 test-doc
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
	@rm -rf target/ast-examples

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

inc-demo:
	@echo "Демонстрация инкрементов/декрементов..."
	@mkdir -p target/ast-examples
	@echo 'fn main() {' > target/ast-examples/inc.src
	@echo '    int x = 5;' >> target/ast-examples/inc.src
	@echo '    int a = x++;  // post-increment' >> target/ast-examples/inc.src
	@echo '    int b = ++x;  // pre-increment' >> target/ast-examples/inc.src
	@echo '    int c = x--;  // post-decrement' >> target/ast-examples/inc.src
	@echo '    int d = --x;  // pre-decrement' >> target/ast-examples/inc.src
	@echo '    int e = x++ + ++x;  // mixed' >> target/ast-examples/inc.src
	@echo '    return x;' >> target/ast-examples/inc.src
	@echo '}' >> target/ast-examples/inc.src
	@echo ""
	@echo "Демонстрационный код:"
	@cat target/ast-examples/inc.src
	@echo ""
	@echo "Лексический анализ (поиск токенов ++ и --):"
	./target/debug/minic lex --input target/ast-examples/inc.src --verbose | grep -E "PLUS_PLUS|MINUS_MINUS" | head -5
	@echo ""
	@echo "Синтаксический анализ (AST с инкрементами):"
	./target/debug/minic parse --input target/ast-examples/inc.src

ll1-demo:
	@echo "Демонстрация LL(1) анализа грамматики..."
	./target/debug/minic ll1 --show-first --show-follow

full-pipeline:
	@echo "Демонстрация полного пайплайна..."
	@mkdir -p target/ast-examples
	@echo '#define MAX 100' > target/ast-examples/full.src
	@echo '#define DEBUG 1' >> target/ast-examples/full.src
	@echo '' >> target/ast-examples/full.src
	@echo 'fn main() {' >> target/ast-examples/full.src
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
	@echo "Препроцессор:"
	@echo "  ./target/debug/minic preprocess --input examples/test.src --defines DEBUG=1 --show"
	@echo "  ./target/debug/minic preprocess --input examples/test.src --preserve-lines --output processed.src"
	@echo ""
	@echo "Полный пайплайн:"
	@echo "  ./target/debug/minic full --input examples/factorial.src --ast-format text"
	@echo "  ./target/debug/minic full --input examples/test.src --defines DEBUG=1 --ast-format dot --output full.dot"
	@echo "  ./target/debug/minic full --input examples/test.src --show-metrics"
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
	@echo "Проверка синтаксиса:"
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
	@mkdir -p examples tests/lexer/valid tests/lexer/invalid tests/parser/valid tests/parser/invalid docs

	# Базовые примеры
	@echo 'fn main() { return 42; }' > examples/hello.src
	@echo 'int x = 10;' > examples/simple.src
	@echo '// Тестовая программа' > examples/test.src

	# Примеры с инкрементами
	@echo 'fn main() { int x = 5; x++; ++x; x--; --x; return x; }' > examples/increment.src
	@echo 'fn compute() { int a = 5; int b = a++ + ++a; return b; }' > examples/complex_inc.src

	# Примеры для парсера
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } return n * factorial(n - 1); }' > examples/factorial.src
	@echo 'struct Point { int x; int y; } fn main() { struct Point p; p.x = 10; p.y = 20; p.x++; return p.x + p.y; }' > examples/struct.src

	# Тесты для парсера
	@echo 'fn add(int a, int b) -> int { return a + b; }' > tests/parser/valid/function.src
	@echo 'if (x > 0) { return 1; } else { return 0; }' > tests/parser/valid/if.src
	@echo 'while (i < 10) { i = i + 1; }' > tests/parser/valid/while.src
	@echo 'for (int i = 0; i < 10; i = i + 1) { print(i); }' > tests/parser/valid/for.src

	# Файлы с ошибками
	@echo 'fn buggy() { int x = 5; x++ return x; }' > examples/errors.src
	@echo 'fn main() { if (x > 0 { return x; } }' >> examples/errors.src

	@echo "Тестовые файлы созданы"

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
	@echo "  make test-integration - Интеграционные тесты"
	@echo "  make test-ll1      - LL(1) тесты"
	@echo "  make test-common   - Тесты общих модулей"
	@echo "  make test-doc      - Документационные тесты"
	@echo "  make test-all      - Все тесты (включая LL(1))"
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
	@echo "Демонстрации (НОВОЕ):"
	@echo "  make ast-demo      - Визуализация AST (text/dot/json)"
	@echo "  make inc-demo      - Демонстрация инкрементов/декрементов"
	@echo "  make error-demo    - Демонстрация восстановления после ошибок"
	@echo "  make ll1-demo      - LL(1) анализ грамматики"
	@echo "  make full-pipeline - Полный пайплайн (препроцессор → лексер → парсер)"
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