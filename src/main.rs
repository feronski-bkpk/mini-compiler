//! CLI интерфейс для Mini Compiler.
//!
//! Предоставляет командную строку для работы с компилятором:
//! - Лексический анализ исходного кода
//! - Синтаксический анализ и построение AST
//! - Препроцессор с макросами
//! - LL(1) анализ грамматики
//! - Метрики ошибок и восстановление
//!
//! # Команды
//!
//! ## Основные команды
//!
//! ```bash
//! # Лексический анализ файла
//! minic lex --input program.src
//!
//! # Синтаксический анализ с выводом AST
//! minic parse --input program.src --ast-format text
//! minic parse --input program.src --ast-format dot --output ast.dot
//! minic parse --input program.src --ast-format json --output ast.json
//!
//! # Препроцессор
//! minic preprocess --input program.src --output processed.src --show
//! minic preprocess --input program.src --defines "DEBUG=1" "VERSION=2"
//!
//! # Полный пайплайн
//! minic full --input program.src --ast-format dot --output ast.dot
//!
//! # LL(1) анализ грамматики
//! minic ll1 --grammar docs/grammar.md
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

use clap::{Parser, Subcommand, ValueEnum};
use minic::lexer::LexerErrorExt;
use minic::parser::{DotGenerator, JsonGenerator, PrettyPrinter};
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

/// Форматы вывода AST.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum AstFormat {
    /// Человекочитаемый текстовый формат
    Text,
    /// Graphviz DOT формат для визуализации
    Dot,
    /// JSON формат для машинной обработки
    Json,
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

    /// Выполнить синтаксический анализ и построить AST
    Parse {
        /// Входной файл с исходным кодом
        #[arg(short, long)]
        input: PathBuf,

        /// Выходной файл для AST
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Формат вывода AST
        #[arg(long, value_enum, default_value_t = AstFormat::Text)]
        ast_format: AstFormat,

        /// Определить макросы для препроцессора
        #[arg(short = 'D', long)]
        defines: Vec<String>,

        /// Применить препроцессор перед парсингом
        #[arg(long)]
        preprocess: bool,

        /// Показать метрики ошибок
        #[arg(long)]
        show_metrics: bool,
    },

    /// Выполнить семантический анализ
    Semantic {
        /// Входной файл
        #[arg(short, long)]
        input: PathBuf,

        /// Выходной файл для результатов
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Показать дамп таблицы символов
        #[arg(long)]
        show_symbols: bool,

        /// Показать декорированное AST
        #[arg(long)]
        show_ast: bool,

        /// Показать размещение в памяти (смещения и размеры)
        #[arg(long)]
        show_layout: bool,
    },

    /// Генерация промежуточного представления (IR)
    Ir {
        /// Входной файл с исходным кодом
        #[arg(short, long)]
        input: PathBuf,

        /// Выходной файл для IR
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Формат вывода IR (text, dot, json)
        #[arg(long, value_enum, default_value_t = AstFormat::Text)]
        ir_format: AstFormat,

        /// Показать статистику IR
        #[arg(long)]
        stats: bool,

        /// Применить оптимизации
        #[arg(long)]
        optimize: bool,

        /// Определить макросы для препроцессора
        #[arg(short = 'D', long)]
        defines: Vec<String>,
    },

    /// Проверить синтаксис исходного кода
    Check {
        /// Входной файл с исходным кодом
        #[arg(short, long)]
        input: PathBuf,

        /// Строгий режим (любая ошибка = failure)
        #[arg(short, long)]
        strict: bool,

        /// Определить макросы для препроцессора
        #[arg(short = 'D', long)]
        defines: Vec<String>,

        /// Применить препроцессор перед проверкой
        #[arg(long)]
        preprocess: bool,
    },

    /// Запустить тесты
    Test {
        /// Запускать только unit-тесты
        #[arg(long)]
        unit: bool,

        /// Запускать только интеграционные тесты
        #[arg(long)]
        integration: bool,

        /// Запускать LL(1) тесты
        #[arg(long)]
        ll1: bool,

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

    /// Полный пайплайн: препроцессор + лексический анализ + синтаксический анализ
    Full {
        /// Входной файл
        #[arg(short, long)]
        input: PathBuf,

        /// Определить макросы
        #[arg(short = 'D', long)]
        defines: Vec<String>,

        /// Формат вывода AST
        #[arg(long, value_enum, default_value_t = AstFormat::Text)]
        ast_format: AstFormat,

        /// Выходной файл для AST
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Показать метрики ошибок
        #[arg(long)]
        show_metrics: bool,
    },

    /// Выполнить LL(1) анализ грамматики
    Ll1 {
        /// Файл с грамматикой (опционально)
        #[arg(short, long)]
        grammar: Option<PathBuf>,

        /// Показать First множества
        #[arg(long)]
        show_first: bool,

        /// Показать Follow множества
        #[arg(long)]
        show_follow: bool,
    },

    /// Демонстрация восстановления после ошибок
    ErrorDemo {
        /// Входной файл с ошибками
        #[arg(short, long)]
        input: PathBuf,

        /// Максимальное количество ошибок
        #[arg(long, default_value_t = 50)]
        max_errors: usize,
    },

    /// Демонстрация инкрементов/декрементов
    IncDemo,
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

        Commands::Parse {
            input,
            output,
            ast_format,
            defines,
            preprocess,
            show_metrics,
        } => handle_parse_command(
            &input,
            output,
            ast_format,
            defines,
            preprocess,
            show_metrics,
            cli.verbose,
        ),

        Commands::Semantic {
            input,
            output,
            show_symbols,
            show_ast,
            show_layout,
        } => handle_semantic_command(
            &input,
            output,
            show_symbols,
            show_ast,
            show_layout,
            cli.verbose,
        ),

        Commands::Ir {
            input,
            output,
            ir_format,
            stats,
            optimize,
            defines,
        } => handle_ir_command(
            &input,
            output,
            ir_format,
            stats,
            optimize,
            defines,
            cli.verbose,
        ),

        Commands::Check {
            input,
            strict,
            defines,
            preprocess,
        } => handle_check_command(&input, strict, defines, preprocess, cli.verbose),

        Commands::Test {
            unit,
            integration,
            ll1,
            coverage,
            test_file,
        } => handle_test_command(unit, integration, ll1, coverage, test_file, cli.verbose),

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
            ast_format,
            output,
            show_metrics,
        } => handle_full_command(
            &input,
            defines,
            ast_format,
            output,
            show_metrics,
            cli.verbose,
        ),

        Commands::Ll1 {
            grammar,
            show_first,
            show_follow,
        } => handle_ll1_command(grammar, show_first, show_follow, cli.verbose),

        Commands::ErrorDemo { input, max_errors } => {
            handle_error_demo(&input, max_errors, cli.verbose)
        }

        Commands::IncDemo => handle_inc_demo(cli.verbose),
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

/// Обрабатывает команду синтаксического анализа.
fn handle_parse_command(
    input: &Path,
    output: Option<PathBuf>,
    ast_format: AstFormat,
    defines: Vec<String>,
    preprocess: bool,
    show_metrics: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Синтаксический анализ файла: {}", input.display());
        if preprocess {
            println!("Препроцессор включен");
        }
        if !defines.is_empty() {
            println!("Макросы: {:?}", defines);
        }
    }

    let source = utils::read_file_with_limit(input)?;

    let parse_output = if preprocess {
        let defines_vec: Vec<(&str, &str)> = defines
            .iter()
            .filter_map(|d| {
                let parts: Vec<&str> = d.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((parts[0], parts[1]))
                } else {
                    Some((d.as_str(), ""))
                }
            })
            .collect();

        compiler::compile(&source, defines_vec)
    } else {
        compiler::syntactic_analysis(&source)
    };

    if parse_output.has_errors() {
        eprintln!("Найдено ошибок: {}", parse_output.errors.len());
        for error in &parse_output.errors.errors {
            eprintln!("  {}", error);
        }
    } else if verbose {
        println!("Ошибок не найдено");
    }

    if show_metrics && parse_output.has_errors() {
        println!("\nМетрики ошибок:");
        println!("{}", parse_output.errors.metrics);
    }

    if let Some(ast) = &parse_output.ast {
        let output_text = match ast_format {
            AstFormat::Text => {
                let mut printer = PrettyPrinter::new();
                printer.format_program(ast)
            }
            AstFormat::Dot => {
                let mut generator = DotGenerator::new();
                generator.generate(ast)
            }
            AstFormat::Json => {
                let mut generator = JsonGenerator::new();
                generator.to_string_pretty(ast)
            }
        };

        if let Some(output_path) = output {
            utils::write_file(&output_path, &output_text)?;
            if verbose {
                println!("AST записан в: {}", output_path.display());
            }
        } else {
            println!("{}", output_text);
        }

        if parse_output.is_valid() {
            Ok(())
        } else {
            Err("Обнаружены ошибки синтаксического анализа".into())
        }
    } else {
        Err("Не удалось построить AST".into())
    }
}

/// Обрабатывает команду проверки синтаксиса.
fn handle_check_command(
    input: &Path,
    strict: bool,
    defines: Vec<String>,
    preprocess: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Проверка файла: {}", input.display());
    }

    let source = utils::read_file_with_limit(input)?;

    let parse_output = if preprocess {
        let defines_vec: Vec<(&str, &str)> = defines
            .iter()
            .filter_map(|d| {
                let parts: Vec<&str> = d.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((parts[0], parts[1]))
                } else {
                    Some((d.as_str(), ""))
                }
            })
            .collect();

        compiler::compile(&source, defines_vec)
    } else {
        compiler::syntactic_analysis(&source)
    };

    if parse_output.is_valid() {
        println!("Файл синтаксически корректен.");
        Ok(())
    } else {
        eprintln!("Найдено {} ошибок:", parse_output.errors.len());
        for error in &parse_output.errors.errors {
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

/// Обрабатывает команду полного пайплайна
fn handle_full_command(
    input: &Path,
    defines: Vec<String>,
    ast_format: AstFormat,
    output: Option<PathBuf>,
    show_metrics: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Запуск полного пайплайна для файла: {}", input.display());
        if !defines.is_empty() {
            println!("Макросы: {:?}", defines);
        }
    }

    println!("Шаг 1: Препроцессор...");
    let source = utils::read_file_with_limit(input)?;

    let defines_vec: Vec<(&str, &str)> = defines
        .iter()
        .filter_map(|d| {
            let parts: Vec<&str> = d.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0], parts[1]))
            } else {
                Some((d.as_str(), ""))
            }
        })
        .collect();

    let parse_output = compiler::compile(&source, defines_vec);

    if verbose {
        println!("Шаг 2: Лексический анализ...");
        println!("Шаг 3: Синтаксический анализ...");
    }

    if parse_output.has_errors() {
        eprintln!("\nНайдено ошибок: {}", parse_output.errors.len());
        for error in &parse_output.errors.errors {
            eprintln!("  {}", error);
        }
    } else if verbose {
        println!("Ошибок не найдено");
    }

    if show_metrics && parse_output.has_errors() {
        println!("\nМетрики ошибок:");
        println!("{}", parse_output.errors.metrics);
    }

    if let Some(ast) = &parse_output.ast {
        let output_text = match ast_format {
            AstFormat::Text => {
                let mut printer = PrettyPrinter::new();
                printer.format_program(ast)
            }
            AstFormat::Dot => {
                let mut generator = DotGenerator::new();
                generator.generate(ast)
            }
            AstFormat::Json => {
                let mut generator = JsonGenerator::new();
                generator.to_string_pretty(ast)
            }
        };

        if let Some(output_path) = output {
            utils::write_file(&output_path, &output_text)?;
            if verbose {
                println!("Результат записан в: {}", output_path.display());
            }
        } else {
            println!("\nAST:\n{}", output_text);
        }
    }

    if parse_output.is_valid() {
        println!("\nПолный пайплайн завершен успешно!");
        Ok(())
    } else {
        println!("\nПолный пайплайн завершен с ошибками");
        Err("Обнаружены ошибки в процессе компиляции".into())
    }
}

/// Обрабатывает команду запуска тестов.
fn handle_test_command(
    unit: bool,
    integration: bool,
    ll1: bool,
    coverage: bool,
    test_file: Option<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Запуск тестов...");
        if ll1 {
            println!("Включая LL(1) тесты");
        }
        if coverage {
            println!("Отчет о покрытии будет сгенерирован");
        }
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("test");

    if unit && !integration && !ll1 {
        cmd.arg("--lib");
    } else if integration && !unit && !ll1 {
        cmd.arg("--tests");
    } else if ll1 && !unit && !integration {
        cmd.arg("--test").arg("ll1_tests");
    }

    if coverage {
        if verbose {
            println!("Для отчета о покрытии требуется cargo-tarpaulin");
        }
        cmd.arg("--coverage");
    }

    if let Some(file) = test_file {
        cmd.arg("--");
        cmd.arg(file);
    }

    if verbose {
        println!("Выполняется: {:?}", cmd);
    }

    let status = cmd.status()?;

    if status.success() {
        println!("Все тесты пройдены!");
        Ok(())
    } else {
        Err("Тесты не пройдены".into())
    }
}

/// Обрабатывает команду LL(1) анализа
fn handle_ll1_command(
    grammar: Option<PathBuf>,
    show_first: bool,
    show_follow: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use minic::parser::ll1::{FirstFollowCalculator, GrammarSymbol, Production};

    println!("LL(1) анализ грамматики");

    if let Some(grammar_path) = grammar {
        if verbose {
            println!("Чтение грамматики из: {}", grammar_path.display());
        }
        // Здесь можно добавить парсинг грамматики из файла
        println!("Функция в разработке");
    } else {
        // Демонстрационная грамматика выражений
        let productions = vec![
            Production {
                left: "E".to_string(),
                right: vec![
                    GrammarSymbol::NonTerminal("T".to_string()),
                    GrammarSymbol::NonTerminal("E'".to_string()),
                ],
            },
            Production {
                left: "E'".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("+".to_string()),
                    GrammarSymbol::NonTerminal("T".to_string()),
                    GrammarSymbol::NonTerminal("E'".to_string()),
                ],
            },
            Production {
                left: "E'".to_string(),
                right: vec![GrammarSymbol::Epsilon],
            },
            Production {
                left: "T".to_string(),
                right: vec![
                    GrammarSymbol::NonTerminal("F".to_string()),
                    GrammarSymbol::NonTerminal("T'".to_string()),
                ],
            },
            Production {
                left: "T'".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("*".to_string()),
                    GrammarSymbol::NonTerminal("F".to_string()),
                    GrammarSymbol::NonTerminal("T'".to_string()),
                ],
            },
            Production {
                left: "T'".to_string(),
                right: vec![GrammarSymbol::Epsilon],
            },
            Production {
                left: "F".to_string(),
                right: vec![GrammarSymbol::Terminal("id".to_string())],
            },
            Production {
                left: "F".to_string(),
                right: vec![
                    GrammarSymbol::Terminal("(".to_string()),
                    GrammarSymbol::NonTerminal("E".to_string()),
                    GrammarSymbol::Terminal(")".to_string()),
                ],
            },
        ];

        let mut calculator = FirstFollowCalculator::new(productions);

        println!("\nВычисление First множеств...");
        calculator.compute_first();

        if show_first {
            println!("\nFirst множества:");
            for (nt, first) in calculator.first_sets() {
                println!("  First({}) = {{", nt);
                for sym in first {
                    match sym {
                        GrammarSymbol::Terminal(t) => println!("    \"{}\"", t),
                        GrammarSymbol::Epsilon => println!("    ε"),
                        _ => {}
                    }
                }
                println!("  }}");
            }
        }

        println!("\nВычисление Follow множеств...");
        calculator.compute_follow();

        if show_follow {
            println!("\nFollow множества:");
            for (nt, follow) in calculator.follow_sets() {
                println!("  Follow({}) = {{", nt);
                for sym in follow {
                    match sym {
                        GrammarSymbol::Terminal(t) => println!("    \"{}\"", t),
                        GrammarSymbol::EndOfFile => println!("    $"),
                        _ => {}
                    }
                }
                println!("  }}");
            }
        }

        if calculator.is_ll1() {
            println!("\nГрамматика является LL(1)");
        } else {
            println!("\nГрамматика НЕ является LL(1)");
        }
    }

    Ok(())
}

/// Обрабатывает команду демонстрации ошибок
fn handle_error_demo(
    input: &Path,
    max_errors: usize,
    _verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Демонстрация восстановления после ошибок");
    println!("Максимальное количество ошибок: {}", max_errors);

    let source = utils::read_file_with_limit(input)?;
    let (tokens, lex_errors) = compiler::lexical_analysis(&source);

    if !lex_errors.is_empty() {
        println!("\nЛексические ошибки:");
        for error in &lex_errors {
            println!("  {}", error);
        }
    }

    let mut parser = minic::parser::Parser::new(tokens);
    parser = parser.with_max_errors(max_errors);

    let ast = parser.parse();

    println!("\nРезультаты синтаксического анализа:");
    println!(
        "  AST построен: {}",
        if ast.is_some() { "good" } else { "bad" }
    );
    println!("  Ошибок: {}", parser.errors().len());

    if parser.errors().len() > 0 {
        println!("\nОшибки:");
        for error in parser.errors().errors.iter() {
            println!("  {}", error);
        }

        println!("\nМетрики:");
        println!("{}", parser.error_metrics());
    }

    Ok(())
}

/// Обрабатывает команду демонстрации инкрементов
fn handle_inc_demo(_verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Демонстрация инкрементов/декрементов");

    let demo_code = r#"
fn main() {
    int x = 5;

    println("Постфиксный инкремент: x++");
    int a = x++;  // a = 5, x = 6
    println("a = {}, x = {}", a, x);

    println("Префиксный инкремент: ++x");
    int b = ++x;  // b = 7, x = 7
    println("b = {}, x = {}", b, x);

    println("Постфиксный декремент: x--");
    int c = x--;  // c = 7, x = 6
    println("c = {}, x = {}", c, x);

    println("Префиксный декремент: --x");
    int d = --x;  // d = 5, x = 5
    println("d = {}, x = {}", d, x);

    println("Сложное выражение: x++ + ++x");
    int e = x++ + ++x;  // e = 5 + 7 = 12, x = 7
    println("e = {}, x = {}", e, x);

    return 0;
}
"#;

    println!("\nДемонстрационный код:");
    println!("{}", demo_code);

    println!("\nЛексический анализ:");
    let mut scanner = Scanner::new(demo_code);
    let (tokens, errors) = scanner.scan_all();

    println!("  Найдено токенов: {}", tokens.len());
    let inc_tokens = tokens
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                minic::TokenKind::PlusPlus | minic::TokenKind::MinusMinus
            )
        })
        .count();
    println!("  Токенов ++/--: {}", inc_tokens);

    println!("\nСинтаксический анализ:");
    let mut parser = minic::parser::Parser::new(tokens);
    let ast = parser.parse();

    if let Some(program) = ast {
        let mut printer = PrettyPrinter::new();
        println!("{}", printer.format_program(&program));
    }

    if errors.is_empty() && parser.errors().is_empty() {
        println!("\nИнкременты работают корректно!");
    } else {
        println!("\nОбнаружены ошибки");
    }

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

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("doc");
    cmd.arg("--no-deps");

    if open {
        cmd.arg("--open");
    }

    let status = cmd.status()?;

    if status.success() {
        println!("Документация сгенерирована");
        Ok(())
    } else {
        Err("Ошибка при генерации документации".into())
    }
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
        println!("  lex       - Лексический анализ исходного кода");
        println!("  parse     - Синтаксический анализ и построение AST");
        println!("  check     - Проверка синтаксиса");
        println!("  test      - Запуск тестов");
        println!("  docs      - Генерация документации");
        println!("  preprocess- Обработка препроцессором");
        println!("  full      - Полный пайплайн компиляции");
        println!("  ll1       - LL(1) анализ грамматики");
        println!("  error-demo- Демонстрация восстановления после ошибок");
        println!("  inc-demo  - Демонстрация инкрементов/декрементов");
        println!("  info      - Информация о компиляторе");
        println!("  spec      - Спецификация языка");
        println!();
        println!("Форматы вывода:");
        println!("  text      - Текстовый формат (по умолчанию)");
        println!("  json      - JSON формат");
        println!("  minimal   - Только ошибки");
        println!("  verbose   - Подробный вывод");
        println!();
        println!("Форматы AST:");
        println!("  text      - Человекочитаемый текст");
        println!("  dot       - Graphviz DOT для визуализации");
        println!("  json      - JSON для машинной обработки");
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

        if verbose {
            println!("\nСодержимое директории docs/:");
            if let Ok(entries) = fs::read_dir("docs") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  {}", entry.path().display());
                    }
                }
            }
        }
    }

    Ok(())
}

/// Обрабатывает команду препроцессора
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
        if !defines.is_empty() {
            println!("Макросы: {:?}", defines);
        }
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
        println!("\n=== РЕЗУЛЬТАТ ПРЕПРОЦЕССОРА ===");
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

fn handle_semantic_command(
    input: &Path,
    output: Option<PathBuf>,
    show_symbols: bool,
    show_ast: bool,
    show_layout: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Семантический анализ файла: {}", input.display());
    }

    let source = utils::read_file_with_limit(input)?;
    let parse_output = compiler::syntactic_analysis(&source);

    if !parse_output.ast.is_some() {
        eprintln!("Не удалось построить AST из-за синтаксических ошибок");
        return Err("Синтаксические ошибки".into());
    }

    let mut analyzer = minic::semantic::SemanticAnalyzer::new();
    let semantic_output = analyzer.analyze(parse_output.ast.unwrap());

    let mut output_text = String::new();

    if semantic_output.has_errors() {
        output_text.push_str(&semantic_output.errors.to_string());
    } else if verbose {
        output_text.push_str("Семантических ошибок не найдено.\n");
    }

    if show_symbols {
        if show_layout {
            output_text.push_str(&semantic_output.symbol_table.dump_with_layout());
        } else {
            output_text.push_str(&semantic_output.symbol_table.dump());
        }
    }

    if show_ast {
        output_text.push_str("\n=== ДЕКОРИРОВАННОЕ AST ===\n");
        let mut printer = minic::semantic::DecoratedAstPrinter::new()
            .with_types(true)
            .with_symbols(show_symbols);
        if let Some(ast) = &semantic_output.decorated_ast {
            output_text.push_str(&printer.format_program(ast, &semantic_output.symbol_table));
        }
    }

    if let Some(output_path) = output {
        utils::write_file(&output_path, &output_text)?;
        if verbose {
            println!("Результат записан в: {}", output_path.display());
        }
    } else {
        print!("{}", output_text);
    }

    if semantic_output.has_errors() {
        Err("Обнаружены семантические ошибки".into())
    } else {
        Ok(())
    }
}

fn handle_ir_command(
    input: &Path,
    output: Option<PathBuf>,
    format: AstFormat,
    stats: bool,
    optimize: bool,
    defines: Vec<String>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Генерация IR для файла: {}", input.display());
        if optimize {
            println!("Оптимизация включена");
        }
        if !defines.is_empty() {
            println!("Макросы: {:?}", defines);
        }
    }

    let source = utils::read_file_with_limit(input)?;

    let defines_vec: Vec<(&str, &str)> = defines
        .iter()
        .filter_map(|d| {
            let parts: Vec<&str> = d.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0], parts[1]))
            } else {
                Some((d.as_str(), ""))
            }
        })
        .collect();

    let (parse_output, ir_program) = minic::compiler::compile_with_ir(&source, defines_vec);

    if parse_output.has_errors() {
        eprintln!("Найдено ошибок: {}", parse_output.errors.len());
        for error in &parse_output.errors.errors {
            eprintln!("  {}", error);
        }
        return Err("Ошибки при компиляции".into());
    }

    let mut ir_program = ir_program.ok_or("Не удалось сгенерировать IR")?;

    if stats {
        println!("{}", minic::ir::IRPrinter::print_stats(&ir_program));
        return Ok(());
    }

    if optimize {
        if verbose {
            println!("Применение оптимизаций...");
        }
        let report = minic::ir::PeepholeOptimizer::optimize(&mut ir_program);

        if verbose {
            println!("Оптимизации завершены:");
            println!("  Изменений: {}", report.changes_made);
            println!("  Удалено инструкций: {}", report.instructions_removed);
            println!("  Упрощений: {}", report.simplifications_applied);
            println!("  Удалено мертвого кода: {}", report.dead_code_removed);
        }
    }

    let output_text = match format {
        AstFormat::Text => minic::ir::IRPrinter::to_text(&ir_program),
        AstFormat::Dot => minic::ir::IRPrinter::to_dot(&ir_program),
        AstFormat::Json => minic::ir::IRPrinter::to_json(&ir_program)?,
    };

    if let Some(output_path) = output {
        utils::write_file(&output_path, &output_text)?;
        if verbose {
            println!("IR записан в: {}", output_path.display());
        }
    } else {
        println!("{}", output_text);
    }

    Ok(())
}

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
                output.push_str(&format!("  {}\n", token));
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

fn format_verbose_output(
    tokens: &[minic::Token],
    errors: &[minic::LexerError],
    scanner: &Scanner,
) -> String {
    let mut output = String::new();

    output.push_str("=== ДЕТАЛЬНЫЙ ОТЧЕТ ЛЕКСИЧЕСКОГО АНАЛИЗА ===\n\n");

    output.push_str("СТАТИСТИКА:\n");
    output.push_str(&format!("  Всего токенов: {}\n", tokens.len()));
    output.push_str(&format!(
        "  Корректных токенов: {}\n",
        tokens.iter().filter(|t| !t.is_eof()).count()
    ));
    output.push_str(&format!("  Ошибок: {}\n", errors.len()));
    output.push_str(&format!("  Позиция завершения: {}\n", scanner.get_line()));
    output.push_str("\n");

    if !errors.is_empty() {
        output.push_str("ОШИБКИ:\n");
        for (i, error) in errors.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, error));
        }
        output.push_str("\n");
    }

    output.push_str("ТОКЕНЫ:\n");
    for (i, token) in tokens.iter().enumerate() {
        if !token.is_eof() {
            output.push_str(&format!("  {:3}: {}\n", i + 1, token));
        }
    }

    let keywords = tokens.iter().filter(|t| t.is_keyword()).count();
    let literals = tokens.iter().filter(|t| t.is_literal()).count();
    let operators = tokens.iter().filter(|t| t.is_operator()).count();
    let delimiters = tokens.iter().filter(|t| t.is_delimiter()).count();
    let increments = tokens
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                minic::TokenKind::PlusPlus | minic::TokenKind::MinusMinus
            )
        })
        .count();

    output.push_str("\nКАТЕГОРИИ ТОКЕНОВ:\n");
    output.push_str(&format!("  Ключевые слова: {}\n", keywords));
    output.push_str(&format!("  Литералы: {}\n", literals));
    output.push_str(&format!("  Операторы: {}\n", operators));
    output.push_str(&format!("  Разделители: {}\n", delimiters));
    output.push_str(&format!("  Инкременты/декременты: {}\n", increments));

    output
}
