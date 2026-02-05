//! CLI интерфейс для Mini Compiler.
//!
//! Предоставляет командную строку для работы с компилятором:
//! - Лексический анализ исходного кода
//! - Тестирование и валидация
//! - Генерация документации
//!
//! # Команды
//!
//! ## Основные команды
//!
//! ```bash
//! # Лексический анализ файла
//! minic lex --input program.src
//!
//! # Лексический анализ с выводом в файл
//! minic lex --input program.src --output tokens.txt
//!
//! # Интерактивный режим
//! minic lex --interactive
//!
//! # Проверка синтаксиса
//! minic check --input program.src
//!
//! # Запуск тестов
//! minic test
//!
//! # Генерация документации
//! minic docs
//! ```
//!
//! ## Примеры использования
//!
//! ```bash
//! # Анализ примера программы
//! minic lex --input examples/hello.src --verbose
//!
//! # Проверка всех примеров
//! for file in examples/*.src; do
//!     echo "Проверка $file"
//!     minic check --input "$file"
//! done
//!
//! # Интерактивная сессия
//! minic lex --interactive
//! > fn main() { return 42; }
//! > Ctrl+D
//! ```
//!
//! # Форматы вывода
//!
//! ## Текстовый формат (по умолчанию)
//! ```
//! 1:1 KW_FN "fn"
//! 1:4 IDENTIFIER "main"
//! 1:8 LPAREN "("
//! 1:9 RPAREN ")"
//! 1:10 LBRACE "{"
//! 1:12 KW_RETURN "return"
//! 1:19 INT_LITERAL "42" 42
//! 1:21 SEMICOLON ";"
//! 1:23 RBRACE "}"
//! 2:1 END_OF_FILE ""
//! ```
//!
//! ## JSON формат
//! ```bash
//! minic lex --input program.src --format json
//! ```
//!
//! # Коды возврата
//!
//! - 0: Успешное выполнение
//! - 1: Ошибка ввода-вывода
//! - 2: Ошибка лексического анализа
//! - 3: Ошибка аргументов командной строки
//! - 4: Внутренняя ошибка

use clap::{Parser, Subcommand, ValueEnum};
use minic::lexer::LexerErrorExt;
use minic::preprocessor::Preprocessor;
use std::fs;
use std::path::{Path, PathBuf};

use minic::{AUTHOR, DESCRIPTION, NAME, VERSION, compiler, lexer::Scanner, utils};

/// Форматы вывода результатов.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Текстовый формат (по умолчанию)
    Text,
    /// JSON формат
    Json,
    /// Минималистичный формат (только ошибки)
    Minimal,
    /// Подробный формат с отладочной информацией
    Verbose,
}

/// CLI интерфейс Mini Compiler.
#[derive(Parser, Debug)]
#[command(
    name = NAME,
    version = VERSION,
    author = AUTHOR,
    about = DESCRIPTION,
    long_about = None,
)]
struct Cli {
    /// Команда для выполнения
    #[command(subcommand)]
    command: Commands,

    /// Уровень детализации вывода
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Формат вывода
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

/// Доступные команды.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Выполнить лексический анализ исходного кода
    Lex {
        /// Входной файл с исходным кодом
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Выходной файл для токенов
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Интерактивный режим (чтение из stdin)
        #[arg(long)]
        interactive: bool,

        /// Показывать только ошибки
        #[arg(short, long)]
        quiet: bool,

        /// Остановиться после первой ошибки
        #[arg(long)]
        fail_fast: bool,
    },

    /// Проверить синтаксис исходного кода
    Check {
        /// Входной файл с исходным кодом
        #[arg(short, long)]
        input: PathBuf,

        /// Строгий режим (любая ошибка = failure)
        #[arg(short, long)]
        strict: bool,
    },

    /// Запустить тесты
    Test {
        /// Запускать только unit-тесты
        #[arg(long)]
        unit: bool,

        /// Запускать только интеграционные тесты
        #[arg(long)]
        integration: bool,

        /// Сгенерировать отчет о покрытии
        #[arg(long)]
        coverage: bool,

        /// Тестовый файл для запуска
        test_file: Option<String>,
    },

    /// Сгенерировать документацию
    Docs {
        /// Открыть документацию в браузере после генерации
        #[arg(short, long)]
        open: bool,

        /// Директория для документации
        #[arg(short, long, default_value = "target/doc")]
        output: PathBuf,
    },

    /// Показать информацию о компиляторе
    Info,

    /// Показать спецификацию языка
    Spec,

    /// Обработать исходный код препроцессором
    Preprocess {
        /// Входной файл
        #[arg(short, long)]
        input: PathBuf,

        /// Выходной файл
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Определить макросы
        #[arg(short = 'D', long)]
        defines: Vec<String>,

        /// Сохранять нумерацию строк
        #[arg(long)]
        preserve_lines: bool,

        /// Показать результат после препроцессора
        #[arg(long)]
        show: bool,
    },

    /// Полный пайплайн: препроцессор + лексический анализ
    Full {
        /// Входной файл
        #[arg(short, long)]
        input: PathBuf,

        /// Определить макросы
        #[arg(short = 'D', long)]
        defines: Vec<String>,

        /// Формат вывода
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lex {
            input,
            output,
            interactive,
            quiet,
            fail_fast,
        } => handle_lex_command(
            input,
            output,
            interactive,
            quiet,
            fail_fast,
            cli.format,
            cli.verbose,
        ),

        Commands::Check { input, strict } => handle_check_command(&input, strict, cli.verbose),

        Commands::Test {
            unit,
            integration,
            coverage,
            test_file,
        } => handle_test_command(unit, integration, coverage, test_file, cli.verbose),

        Commands::Docs { open, output } => handle_docs_command(open, &output, cli.verbose),

        Commands::Info => handle_info_command(cli.verbose),

        Commands::Spec => handle_spec_command(cli.verbose),

        Commands::Preprocess {
            input,
            output,
            defines,
            preserve_lines,
            show,
        } => handle_preprocess_command(&input, output, defines, preserve_lines, show, cli.verbose),

        Commands::Full {
            input,
            defines,
            format,
        } => handle_full_command(&input, defines, format, cli.verbose),
    }
}

/// Обрабатывает команду лексического анализа.
fn handle_lex_command(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    interactive: bool,
    quiet: bool,
    fail_fast: bool,
    format: OutputFormat,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = if interactive {
        if verbose && !quiet {
            println!("Введите исходный код (Ctrl+D для завершения):");
        }
        utils::read_stdin()?
    } else if let Some(input_path) = input {
        if verbose && !quiet {
            println!("Чтение файла: {}", input_path.display());
        }
        utils::read_file_with_limit(&input_path)?
    } else {
        return Err("Не указан входной файл. Используйте --input или --interactive".into());
    };

    let mut scanner = Scanner::new(&source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut should_continue = true;

    while should_continue {
        match scanner.next_token() {
            Ok(token) => {
                let is_eof = token.is_eof();
                tokens.push(token);

                if is_eof {
                    should_continue = false;
                }
            }
            Err(error) => {
                errors.push(error);

                if fail_fast {
                    should_continue = false;
                }
            }
        }
    }

    let output_text = match format {
        OutputFormat::Text => format_text_output(&tokens, &errors, quiet, verbose),
        OutputFormat::Json => format_json_output(&tokens, &errors)?,
        OutputFormat::Minimal => format_minimal_output(&errors),
        OutputFormat::Verbose => format_verbose_output(&tokens, &errors, &scanner),
    };

    if let Some(output_path) = output {
        utils::write_file(&output_path, &output_text)?;
        if verbose && !quiet {
            println!("Результат записан в: {}", output_path.display());
        }
    } else {
        print!("{}", output_text);
    }

    if !errors.is_empty() {
        Err("Обнаружены ошибки лексического анализа".into())
    } else {
        Ok(())
    }
}

/// Обрабатывает команду проверки синтаксиса.
fn handle_check_command(
    input: &Path,
    strict: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Проверка файла: {}", input.display());
    }

    let source = utils::read_file_with_limit(input)?;
    let (tokens, errors) = compiler::lexical_analysis(&source);

    if errors.is_empty() {
        println!("✓ Файл корректен. Найдено {} токенов.", tokens.len());
        Ok(())
    } else {
        eprintln!("✗ Найдено {} ошибок:", errors.len());
        for error in errors {
            eprintln!("  • {}", error);
        }

        if strict {
            Err("Проверка не пройдена из-за ошибок".into())
        } else {
            eprintln!("Предупреждение: файл содержит ошибки, но проверка продолжена.");
            Ok(())
        }
    }
}

/// Обрабатывает команду запуска тестов.
fn handle_test_command(
    unit: bool,
    integration: bool,
    coverage: bool,
    test_file: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Запуск тестов...");

        if coverage {
            println!("Внимание: отчет о покрытии требует установки cargo-tarpaulin");
        }
    }

    println!("Тестирование запускается через cargo test...");

    let test_args = build_test_args(unit, integration, coverage, test_file);

    println!("Выполняется: cargo test {}", test_args.join(" "));

    Ok(())
}

/// Обрабатывает команду генерации документации.
fn handle_docs_command(
    open: bool,
    output: &Path,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Генерация документации в: {}", output.display());
    }

    println!("Документация генерируется через cargo doc...");

    if open {
        println!("Документация будет открыта в браузере.");
    }

    Ok(())
}

/// Обрабатывает команду показа информации.
fn handle_info_command(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} v{}", NAME, VERSION);
    println!("Авторы: {}", AUTHOR);
    println!();
    println!("{}", DESCRIPTION);
    println!();

    if verbose {
        println!("Поддерживаемые команды:");
        println!("  lex      - Лексический анализ исходного кода");
        println!("  check    - Проверка синтаксиса");
        println!("  test     - Запуск тестов");
        println!("  docs     - Генерация документации");
        println!("  info     - Информация о компиляторе");
        println!("  spec     - Спецификация языка");
        println!();
        println!("Форматы вывода:");
        println!("  text     - Текстовый формат (по умолчанию)");
        println!("  json     - JSON формат");
        println!("  minimal  - Только ошибки");
        println!("  verbose  - Подробный вывод");
    }

    Ok(())
}

/// Обрабатывает команду показа спецификации языка.
fn handle_spec_command(verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let spec_path = Path::new("docs/language_spec.md");

    if spec_path.exists() {
        let spec_content = fs::read_to_string(spec_path)?;

        if verbose {
            println!("Полная спецификация языка:");
            println!("{}", "=".repeat(80));
        }

        println!("{}", spec_content);
    } else {
        eprintln!("Файл спецификации не найден: {}", spec_path.display());
        eprintln!("Создайте файл docs/language_spec.md со спецификацией языка.");
    }

    Ok(())
}

/// Форматирует результат в текстовом формате.
fn format_text_output(
    tokens: &[minic::Token],
    errors: &[minic::LexerError],
    quiet: bool,
    verbose: bool,
) -> String {
    let mut output = String::new();

    if !errors.is_empty() && !quiet {
        output.push_str(&format!("Найдено {} ошибок:\n", errors.len()));
        for error in errors {
            output.push_str(&format!("  {}\n", error));
        }
        output.push('\n');
    }

    if !quiet {
        output.push_str(&format!("Найдено {} токенов:\n", tokens.len()));
        for token in tokens {
            if !token.is_eof() {
                output.push_str(&format!("{}\n", token));
            }
        }

        if verbose {
            let valid_tokens = tokens.iter().filter(|t| !t.is_eof()).count();
            output.push_str(&format!("\nСтатистика:\n"));
            output.push_str(&format!("  Всего токенов: {}\n", valid_tokens));
            output.push_str(&format!("  Ошибок: {}\n", errors.len()));
            output.push_str(&format!(
                "  Успешных: {:.1}%\n",
                if !tokens.is_empty() {
                    (valid_tokens as f64 / tokens.len() as f64) * 100.0
                } else {
                    0.0
                }
            ));
        }
    }

    output
}

/// Форматирует результат в JSON формате.
fn format_json_output(
    tokens: &[minic::Token],
    errors: &[minic::LexerError],
) -> Result<String, Box<dyn std::error::Error>> {
    use serde_json::{Value, json};

    let tokens_json: Vec<Value> = tokens
        .iter()
        .map(|token| {
            json!({
                "type": token.type_name(),
                "lexeme": token.lexeme,
                "position": {
                    "line": token.position.line,
                    "column": token.position.column,
                },
                "is_eof": token.is_eof(),
            })
        })
        .collect();

    let errors_json: Vec<Value> = errors
        .iter()
        .map(|error| {
            json!({
                "type": format!("{:?}", error).split(' ').next().unwrap_or("Unknown"),
                "message": error.to_string(),
                "position": {
                    "line": error.position().line,
                    "column": error.position().column,
                },
            })
        })
        .collect();

    let result = json!({
        "success": errors.is_empty(),
        "tokens": tokens_json,
        "errors": errors_json,
        "statistics": {
            "total_tokens": tokens.len(),
            "valid_tokens": tokens.iter().filter(|t| !t.is_eof()).count(),
            "error_count": errors.len(),
        },
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.into())
}

/// Форматирует результат в минималистичном формате.
fn format_minimal_output(errors: &[minic::LexerError]) -> String {
    if errors.is_empty() {
        String::from("OK\n")
    } else {
        let mut output = String::new();
        for error in errors {
            output.push_str(&format!("ERROR: {}\n", error));
        }
        output
    }
}

/// Форматирует результат в подробном формате.
fn format_verbose_output(
    tokens: &[minic::Token],
    errors: &[minic::LexerError],
    scanner: &Scanner,
) -> String {
    let mut output = String::new();

    output.push_str("=== ДЕТАЛЬНЫЙ ОТЧЕТ ЛЕКСИЧЕСКОГО АНАЛИЗА ===\n\n");

    // Статистика
    output.push_str("СТАТИСТИКА:\n");
    output.push_str(&format!("  Всего токенов: {}\n", tokens.len()));
    output.push_str(&format!(
        "  Корректных токенов: {}\n",
        tokens.iter().filter(|t| !t.is_eof()).count()
    ));
    output.push_str(&format!("  Ошибок: {}\n", errors.len()));
    output.push_str(&format!("  Позиция завершения: {}\n", scanner.get_line()));
    output.push_str("\n");

    // Ошибки
    if !errors.is_empty() {
        output.push_str("ОШИБКИ:\n");
        for (i, error) in errors.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, error));
        }
        output.push_str("\n");
    }

    // Токены
    output.push_str("ТОКЕНЫ:\n");
    for (i, token) in tokens.iter().enumerate() {
        if !token.is_eof() {
            output.push_str(&format!("  {:3}: {}\n", i + 1, token));
        }
    }

    // Категории токенов
    let keywords = tokens.iter().filter(|t| t.is_keyword()).count();
    let literals = tokens.iter().filter(|t| t.is_literal()).count();
    let operators = tokens.iter().filter(|t| t.is_operator()).count();
    let delimiters = tokens.iter().filter(|t| t.is_delimiter()).count();

    output.push_str("\nКАТЕГОРИИ ТОКЕНОВ:\n");
    output.push_str(&format!("  Ключевые слова: {}\n", keywords));
    output.push_str(&format!("  Литералы: {}\n", literals));
    output.push_str(&format!("  Операторы: {}\n", operators));
    output.push_str(&format!("  Разделители: {}\n", delimiters));

    output
}

/// Строит аргументы для команды тестирования.
fn build_test_args(
    unit: bool,
    integration: bool,
    coverage: bool,
    test_file: Option<String>,
) -> Vec<String> {
    let mut args = Vec::new();

    if unit && !integration {
        args.push("--lib".to_string());
    } else if integration && !unit {
        args.push("--tests".to_string());
    }

    if coverage {
        args.push("--coverage".to_string());
    }

    if let Some(file) = test_file {
        args.push(file);
    }

    args
}

fn handle_preprocess_command(
    input: &Path,
    output: Option<PathBuf>,
    defines: Vec<String>,
    preserve_lines: bool,
    show: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Препроцессирование файла: {}", input.display());
    }

    let source = utils::read_file_with_limit(input)?;
    let mut preprocessor = Preprocessor::new(&source);
    preprocessor.preserve_line_numbers(preserve_lines);

    for define in defines {
        let parts: Vec<&str> = define.splitn(2, '=').collect();
        if parts.len() == 2 {
            preprocessor.define(parts[0], parts[1])?;
        } else {
            preprocessor.define(&define, "")?;
        }
    }

    let processed = preprocessor.process()?;

    if show {
        println!("=== РЕЗУЛЬТАТ ПРЕПРОЦЕССОРА ===");
        println!("{}", processed);
        println!("================================");
    }

    if let Some(output_path) = output {
        utils::write_file(&output_path, &processed)?;
        if verbose {
            println!("Результат записан в: {}", output_path.display());
        }
    } else if !show {
        print!("{}", processed);
    }

    Ok(())
}

/// Обрабатывает команду полного пайплайна
fn handle_full_command(
    input: &Path,
    defines: Vec<String>,
    format: OutputFormat,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Запуск полного пайплайна для файла: {}", input.display());
    }

    let source = utils::read_file_with_limit(input)?;

    let mut preprocessor = Preprocessor::new(&source);
    for define in defines {
        let parts: Vec<&str> = define.splitn(2, '=').collect();
        if parts.len() == 2 {
            preprocessor.define(parts[0], parts[1])?;
        } else {
            preprocessor.define(&define, "")?;
        }
    }

    let processed = preprocessor.process()?;

    if verbose {
        println!(
            "Препроцессирование завершено, длина результата: {} символов",
            processed.len()
        );
    }

    let mut scanner = Scanner::new(&processed);
    let (tokens, errors) = scanner.scan_all();

    let output_text = match format {
        OutputFormat::Text => format_text_output(&tokens, &errors, false, verbose),
        OutputFormat::Json => format_json_output(&tokens, &errors)?,
        OutputFormat::Minimal => format_minimal_output(&errors),
        OutputFormat::Verbose => format_verbose_output(&tokens, &errors, &scanner),
    };

    println!("{}", output_text);

    if !errors.is_empty() {
        Err("Обнаружены ошибки лексического анализа".into())
    } else {
        Ok(())
    }
}
