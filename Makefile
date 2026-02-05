.PHONY: all build release check test clean docs help

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

test:
	@echo "Запуск тестов..."
	$(CARGO) test -- --nocapture

test-lexer:
	@echo "Запуск тестов лексического анализатора..."
	$(CARGO) test lexer -- --nocapture

test-common:
	@echo "Запуск тестов общих модулей..."
	$(CARGO) test common -- --nocapture

lint:
	@echo "Проверка линтером..."
	$(CARGO) clippy -- -D warnings

fmt:
	@echo "Форматирование кода..."
	$(CARGO) fmt

docs:
	@echo "Генерация документации..."
	$(CARGO) doc --no-deps --open

clean:
	@echo "Очистка..."
	$(CARGO) clean

udeps:
	@echo "Анализ неиспользуемых зависимостей..."
	$(CARGO) udeps

bench:
	@echo "Запуск бенчмарков..."
	$(CARGO) bench

coverage:
	@echo "Измерение покрытия кода..."
	cargo tarpaulin --ignore-tests --out Html

example:
	@echo "Пример использования:"
	@echo "  ./target/debug/minic lex --input examples/hello.src --verbose"
	@echo "  ./target/debug/minic check --input examples/hello.src"
	@echo "  ./target/debug/minic test"

create-test-files:
	@echo "Создание тестовых файлов..."
	@mkdir -p examples tests/lexer/valid tests/lexer/invalid docs
	@echo 'fn main() { return 42; }' > examples/hello.src
	@echo 'int x = 10;' > examples/simple.src
	@echo '// Тестовая программа' > examples/test.src

help:
	@echo "Mini Compiler - система сборки"
	@echo ""
	@echo "Доступные команды:"
	@echo "  build          - Сборка проекта"
	@echo "  release        - Сборка в режиме релиза"
	@echo "  check          - Проверка кода без сборки"
	@echo "  test           - Запуск всех тестов"
	@echo "  test-lexer     - Запуск тестов лексического анализатора"
	@echo "  test-common    - Запуск тестов общих модулей"
	@echo "  lint           - Проверка линтером"
	@echo "  fmt            - Форматирование кода"
	@echo "  docs           - Генерация документации"
	@echo "  clean          - Очистка"
	@echo "  udeps          - Анализ неиспользуемых зависимостей"
	@echo "  bench          - Запуск бенчмарков"
	@echo "  coverage       - Измерение покрытия кода"
	@echo "  example        - Показать примеры использования"
	@echo "  create-test-files - Создать тестовые файлы"
	@echo "  help           - Показать эту справку"
	@echo ""
	@echo "Примеры:"
	@echo "  make build test"
	@echo "  make release && ./target/release/minic --help"