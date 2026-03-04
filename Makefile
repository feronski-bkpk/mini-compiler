.PHONY: all build release check test test-parser test-preprocessor test-integration clean docs help run-example ast-demo

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

test-common:
	@echo "Запуск тестов общих модулей..."
	$(CARGO) test common -- --nocapture

test-doc:
	@echo "Запуск документационных тестов..."
	$(CARGO) test --doc

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
	@echo ""
	@echo "Препроцессор:"
	@echo "  ./target/debug/minic preprocess --input examples/test.src --defines DEBUG=1 --show"
	@echo "  ./target/debug/minic preprocess --input examples/test.src --preserve-lines --output processed.src"
	@echo ""
	@echo "Полный пайплайн:"
	@echo "  ./target/debug/minic full --input examples/factorial.src --ast-format text"
	@echo "  ./target/debug/minic full --input examples/test.src --defines DEBUG=1 --ast-format dot --output full.dot"
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

ast-demo:
	@echo "Демонстрация визуализации AST..."
	@mkdir -p target/ast-examples
	@echo "fn main() { return 42; }" > target/ast-examples/simple.src
	@echo "struct Point { int x; int y; } fn main() { struct Point p; p.x = 10; return p.x; }" > target/ast-examples/struct.src
	@echo ""
	@echo "Текстовый формат AST:"
	./target/debug/minic parse --input target/ast-examples/simple.src
	@echo ""
	@echo "Генерация DOT графа..."
	./target/debug/minic parse --input target/ast-examples/struct.src --ast-format dot --output target/ast-examples/ast.dot
	@echo "DOT файл сохранен: target/ast-examples/ast.dot"
	@echo "Для визуализации выполните: dot -Tpng target/ast-examples/ast.dot -o target/ast-examples/ast.png"
	@echo ""
	@echo "JSON формат:"
	./target/debug/minic parse --input target/ast-examples/simple.src --ast-format json

create-test-files:
	@echo "Создание тестовых файлов..."
	@mkdir -p examples tests/lexer/valid tests/lexer/invalid tests/parser/valid tests/parser/invalid docs

	# Базовые примеры
	@echo 'fn main() { return 42; }' > examples/hello.src
	@echo 'int x = 10;' > examples/simple.src
	@echo '// Тестовая программа' > examples/test.src

	# Примеры для парсера
	@echo 'fn factorial(int n) -> int { if (n <= 1) { return 1; } return n * factorial(n - 1); }' > examples/factorial.src
	@echo 'struct Point { int x; int y; } fn main() { struct Point p; p.x = 10; p.y = 20; return p.x + p.y; }' > examples/struct.src

	# Тесты для парсера
	@echo 'fn add(int a, int b) -> int { return a + b; }' > tests/parser/valid/function.src
	@echo 'if (x > 0) { return 1; } else { return 0; }' > tests/parser/valid/if.src
	@echo 'while (i < 10) { i = i + 1; }' > tests/parser/valid/while.src
	@echo 'for (int i = 0; i < 10; i = i + 1) { print(i); }' > tests/parser/valid/for.src

	@echo "Тестовые файлы созданы"

# === Утилиты ===
graphviz-check:
	@echo "Проверка наличия Graphviz..."
	@command -v dot >/dev/null 2>&1 && echo "Graphviz установлен" || echo "Graphviz не установлен. Установите: https://graphviz.org/download/"

install-deps:
	@echo "Установка зависимостей для разработки..."
	@cargo install cargo-udeps || true
	@cargo install cargo-tarpaulin || true

# === Справка ===
help:
	@echo "Mini Compiler - система сборки (Спринт 2)"
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
	@echo "  make test-common   - Тесты общих модулей"
	@echo "  make test-doc      - Документационные тесты"
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
	@echo "Демонстрация:"
	@echo "  make example       - Показать примеры использования"
	@echo "  make ast-demo      - Демонстрация визуализации AST"
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
	@echo "Примеры использования:"
	@echo "  make build test"
	@echo "  make release && ./target/release/minic --help"
	@echo "  make ast-demo      # Визуализация AST"
	@echo ""